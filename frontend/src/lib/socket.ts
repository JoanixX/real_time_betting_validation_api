import { WS_BASE_URL } from "./constants";
import type { WSEvent, WSEventType, WSEventPayloadMap } from "@/types/domain";

// configuración del cliente WS
interface SocketConfig {
  url: string;
  // Reconexión exponencial
  reconnect: boolean;
  reconnectBaseDelay: number; // ms — primer reintento
  reconnectMaxDelay: number; // ms — tope del backoff
  reconnectMaxAttempts: number;
  // heartbeat
  heartbeatInterval: number; // ms — cada cuánto mandamos ping
  heartbeatTimeout: number; // ms — cuánto esperamos el pong
}

const DEFAULT_CONFIG: SocketConfig = {
  url: WS_BASE_URL,
  reconnect: true,
  reconnectBaseDelay: 1_000,
  reconnectMaxDelay: 30_000,
  reconnectMaxAttempts: 15,
  heartbeatInterval: 30_000,
  heartbeatTimeout: 5_000,
};

// callbacks del ciclo de vida
type ConnectionHandler = () => void;
type ErrorHandler = (error: Event) => void;
type EventCallback<T = unknown> = (payload: T) => void;

// estado de conexión observable
export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "connected"
  | "reconnecting";

// se envia mensaje encolado mientras el socket está desconectado
interface QueuedMessage {
  data: string;
  timestamp: number;
}

/**
 * clienet WebSocket nativo con:
 * - reconexion exponencial con jitter
 * - Message queueing para envíos durante desconexión
 * - Heartbeat para detectar conexiones zombies
 * - event emitter tipado estricto
 *
 * ejemplo: `const ws = NativeSocketClient.getInstance()`
 */
export class NativeSocketClient {
  private static instance: NativeSocketClient | null = null;

  private ws: WebSocket | null = null;
  private config: SocketConfig;
  private state: ConnectionState = "disconnected";
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private heartbeatTimeoutTimer: ReturnType<typeof setTimeout> | null = null;

  // mensajes que se envían al reconectar
  private messageQueue: QueuedMessage[] = [];
  private readonly maxQueueSize = 100;

  // event listeners tipados — Map<evento, Set<callbacks>>
  private eventListeners = new Map<string, Set<EventCallback>>();
  // listeners de ciclo de vida
  private onConnectHandlers = new Set<ConnectionHandler>();
  private onDisconnectHandlers = new Set<ConnectionHandler>();
  private onErrorHandlers = new Set<ErrorHandler>();
  private onStateChangeHandlers = new Set<EventCallback<ConnectionState>>();

  private constructor(config: Partial<SocketConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  // Singleton
  static getInstance(config?: Partial<SocketConfig>): NativeSocketClient {
    if (!NativeSocketClient.instance) {
      NativeSocketClient.instance = new NativeSocketClient(config);
    }
    return NativeSocketClient.instance;
  }

  // Para tests — destruye la instancia
  static resetInstance(): void {
    NativeSocketClient.instance?.disconnect();
    NativeSocketClient.instance = null;
  }

  // conexión
  connect(): void {
    if (
      this.ws?.readyState === WebSocket.OPEN ||
      this.ws?.readyState === WebSocket.CONNECTING
    ) {
      return;
    }

    this.setState("connecting");
    this.ws = new WebSocket(this.config.url);

    this.ws.onopen = () => {
      this.setState("connected");
      this.reconnectAttempts = 0;
      this.flushQueue();
      this.startHeartbeat();
      this.onConnectHandlers.forEach((h) => h());
    };

    this.ws.onclose = () => {
      this.stopHeartbeat();
      this.onDisconnectHandlers.forEach((h) => h());

      if (
        this.config.reconnect &&
        this.reconnectAttempts < this.config.reconnectMaxAttempts
      ) {
        this.scheduleReconnect();
      } else {
        this.setState("disconnected");
      }
    };

    this.ws.onerror = (event) => {
      this.onErrorHandlers.forEach((h) => h(event));
    };

    this.ws.onmessage = (event) => {
      this.handleMessage(event.data);
    };
  }

  disconnect(): void {
    this.config.reconnect = false; // evita que reconecte al cerrar
    this.clearReconnectTimer();
    this.stopHeartbeat();

    if (this.ws) {
      this.ws.onclose = null; // evita trigger de reconexión
      this.ws.close(1000, "Client disconnect");
      this.ws = null;
    }

    this.setState("disconnected");
    // Re-habilitar para futuras conexiones
    this.config.reconnect = true;
  }

  // envío de mensajes
  send(type: string, payload: unknown): void {
    const message = JSON.stringify({ type, payload });

    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(message);
    } else {
      this.enqueue(message);
    }
  }

  // event emitter tipado
  on<T extends WSEventType>(
    event: T,
    callback: EventCallback<WSEventPayloadMap[T]>,
  ): () => void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    const listeners = this.eventListeners.get(event)!;
    listeners.add(callback as EventCallback);

    // Retorna unsubscribe
    return () => {
      listeners.delete(callback as EventCallback);
      if (listeners.size === 0) {
        this.eventListeners.delete(event);
      }
    };
  }

  // suscribirse a TODOS los eventos (para debug/logging)
  onAny(callback: EventCallback<WSEvent>): () => void {
    return this.on("*" as WSEventType, callback as EventCallback);
  }

  // listeners de ciclo de vida
  onConnect(handler: ConnectionHandler): () => void {
    this.onConnectHandlers.add(handler);
    return () => this.onConnectHandlers.delete(handler);
  }

  onDisconnect(handler: ConnectionHandler): () => void {
    this.onDisconnectHandlers.add(handler);
    return () => this.onDisconnectHandlers.delete(handler);
  }

  onError(handler: ErrorHandler): () => void {
    this.onErrorHandlers.add(handler);
    return () => this.onErrorHandlers.delete(handler);
  }

  onStateChange(handler: EventCallback<ConnectionState>): () => void {
    this.onStateChangeHandlers.add(handler);
    return () => this.onStateChangeHandlers.delete(handler);
  }

  // getters
  get connectionState(): ConnectionState {
    return this.state;
  }

  get isConnected(): boolean {
    return this.state === "connected";
  }

  get queueSize(): number {
    return this.messageQueue.length;
  }

  // internos
  // parsea el mensaje y routea al listener correcto
  private handleMessage(raw: string): void {
    // pong del heartbeat
    if (raw === "pong") {
      this.clearHeartbeatTimeout();
      return;
    }

    try {
      const event = JSON.parse(raw) as WSEvent;

      // Listeners específicos del evento
      const listeners = this.eventListeners.get(event.type);
      if (listeners) {
        listeners.forEach((cb) => cb(event.payload));
      }

      // listeners wildcard
      const wildcardListeners = this.eventListeners.get("*");
      if (wildcardListeners) {
        wildcardListeners.forEach((cb) => cb(event));
      }
    } catch {
      console.warn("[WS] Mensaje no parseable:", raw);
    }
  }

  // reconexión con exponential backoff + jitter
  private scheduleReconnect(): void {
    this.setState("reconnecting");
    this.reconnectAttempts++;

    // delay = base * 2^attempt + jitter random
    const exponentialDelay =
      this.config.reconnectBaseDelay * Math.pow(2, this.reconnectAttempts - 1);
    const jitter = Math.random() * this.config.reconnectBaseDelay;
    const delay = Math.min(
      exponentialDelay + jitter,
      this.config.reconnectMaxDelay,
    );

    console.info(
      `[WS] Reconectando en ${Math.round(delay)}ms (intento ${this.reconnectAttempts}/${this.config.reconnectMaxAttempts})`,
    );

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.reconnectAttempts = 0;
  }

  // heartbeat que detecta conexiones zombies
  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send("ping");
        // si no recibimos pong, la conexión está muerta
        this.heartbeatTimeoutTimer = setTimeout(() => {
          console.warn("[WS] Heartbeat timeout — cerrando conexión zombie");
          this.ws?.close(4000, "Heartbeat timeout");
        }, this.config.heartbeatTimeout);
      }
    }, this.config.heartbeatInterval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
    this.clearHeartbeatTimeout();
  }

  private clearHeartbeatTimeout(): void {
    if (this.heartbeatTimeoutTimer) {
      clearTimeout(this.heartbeatTimeoutTimer);
      this.heartbeatTimeoutTimer = null;
    }
  }

  // Message queue
  private enqueue(data: string): void {
    if (this.messageQueue.length >= this.maxQueueSize) {
      // Dropeamos el más viejo
      this.messageQueue.shift();
    }
    this.messageQueue.push({ data, timestamp: Date.now() });
  }

  private flushQueue(): void {
    if (
      this.messageQueue.length === 0 ||
      this.ws?.readyState !== WebSocket.OPEN
    )
      return;

    console.info(
      `[WS] Flushing ${this.messageQueue.length} mensajes encolados`,
    );
    const queue = [...this.messageQueue];
    this.messageQueue = [];

    for (const msg of queue) {
      // descartamos mensajes de más de 60s
      if (Date.now() - msg.timestamp < 60_000) {
        this.ws!.send(msg.data);
      }
    }
  }

  // actualiza estado y notifica
  private setState(newState: ConnectionState): void {
    if (this.state === newState) return;
    this.state = newState;
    this.onStateChangeHandlers.forEach((h) => h(newState));
  }
}

// exportamos el singleton helper
export function getSocketClient(
  config?: Partial<SocketConfig>,
): NativeSocketClient {
  return NativeSocketClient.getInstance(config);
}