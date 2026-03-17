// refleja backend/src/domain/models.rs
import { BET_STATUS, MATCH_STATUS, WS_EVENTS } from '@/lib/constants';

// Apuestas
export interface BetTicket {
  user_id: string;    // UUID como string
  match_id: string;   // UUID como string
  amount: number;     // f64
  odds: number;       // f64
}

export type BetStatus = typeof BET_STATUS[keyof typeof BET_STATUS];

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
  | typeof WS_EVENTS.BET_ACCEPTED
  | typeof WS_EVENTS.BET_REJECTED
  | typeof WS_EVENTS.ODDS_UPDATED
  | typeof WS_EVENTS.MATCH_STATUS_CHANGED;

// Payload compartido de eventos de apuesta
export interface BetEventPayload {
  bet_id: string;
  user_id: string;
  match_id: string;
  amount: number;
  odds: number;
  status: BetStatus;
}

export interface WSBetAcceptedEvent {
  type: typeof WS_EVENTS.BET_ACCEPTED;
  payload: BetEventPayload;
}

export interface WSBetRejectedEvent {
  type: typeof WS_EVENTS.BET_REJECTED;
  payload: BetEventPayload;
}

export interface WSOddsUpdate {
  type: typeof WS_EVENTS.ODDS_UPDATED;
  payload: {
    match_id: string;
    odds: number;
    timestamp: number;
  };
}

export type MatchStatus = typeof MATCH_STATUS[keyof typeof MATCH_STATUS];

export interface Match {
  id: string;
  home_team: string;
  away_team: string;
  status: MatchStatus;
  odds: number; // odds iniciales
}

export interface WSMatchStatusEvent {
  type: typeof WS_EVENTS.MATCH_STATUS_CHANGED;
  payload: {
    match_id: string;
    status: MatchStatus;
  };
}

export type WSEvent = WSBetAcceptedEvent | WSBetRejectedEvent | WSOddsUpdate | WSMatchStatusEvent;

// mapa de evento que va a ser el payload para tipado estricto del event emitter
export interface WSEventPayloadMap {
  [WS_EVENTS.BET_ACCEPTED]: BetEventPayload;
  [WS_EVENTS.BET_REJECTED]: BetEventPayload;
  [WS_EVENTS.ODDS_UPDATED]: WSOddsUpdate['payload'];
  [WS_EVENTS.MATCH_STATUS_CHANGED]: WSMatchStatusEvent['payload'];
}