// refleja backend/src/domain/models.rs

// Apuestas
export interface BetTicket {
  user_id: string;    // UUID como string
  match_id: string;   // UUID como string
  amount: number;     // f64
  odds: number;       // f64
}

export type BetStatus = 'Pending' | 'Validated' | 'Rejected';

export interface ValidateBetRequest {
  user_id: string;
  match_id: string;
  amount: number;
  odds: number;
}

export type ValidateBetResponse = BetTicket;

// guardamos el historial de apuestas (REST)
export interface BetHistoryEntry {
  bet_id: string;
  user_id: string;
  match_id: string;
  amount: number;
  odds: number;
  status: BetStatus;
  created_at: string; // ISO 8601
}

// respuesta al colocar apuesta (REST)
export interface PlaceBetResponse {
  bet_id: string;
  status: BetStatus;
}

// Usuarios
export interface User {
  id: string;
  email: string;
  name: string;
  created_at: string; // ISO 8601
}

export interface CreateUserRequest {
  email: string;
  password: string;
  name: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  status: 'created' | 'authenticated';
  user_id: string;
  name?: string;
}

// Sistema
export interface HealthCheckResponse {
  status: 'ok';
}

export interface ActivityLogEntry {
  timestamp: string;
  amount: number;
  latency_ms: number;
  status?: BetStatus;
}

// Eventos de Websocket
export type WSEventType =
  | 'bet:validated'
  | 'bet:rejected'
  | 'odds:updated'
  | 'match:status_changed';

// Payload compartido de eventos de apuesta
export interface BetEventPayload {
  bet_id: string;
  user_id: string;
  match_id: string;
  amount: number;
  odds: number;
  status: BetStatus;
}

export interface WSBetValidatedEvent {
  type: 'bet:validated';
  payload: BetEventPayload;
}

export interface WSBetRejectedEvent {
  type: 'bet:rejected';
  payload: BetEventPayload;
}

export interface WSOddsUpdate {
  type: 'odds:updated';
  payload: {
    match_id: string;
    odds: number;
    timestamp: number;
  };
}

export type MatchStatus = 'upcoming' | 'live' | 'finished' | 'suspended';

export interface WSMatchStatusEvent {
  type: 'match:status_changed';
  payload: {
    match_id: string;
    status: MatchStatus;
  };
}

export type WSEvent = WSBetValidatedEvent | WSBetRejectedEvent | WSOddsUpdate | WSMatchStatusEvent;

// mapa de evento que va a ser el payload para tipado estricto del event emitter
export interface WSEventPayloadMap {
  'bet:validated': BetEventPayload;
  'bet:rejected': BetEventPayload;
  'odds:updated': WSOddsUpdate['payload'];
  'match:status_changed': WSMatchStatusEvent['payload'];
}
