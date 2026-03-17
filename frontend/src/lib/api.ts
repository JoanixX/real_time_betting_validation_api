import axios from 'axios';
import { API_BASE_URL, ENDPOINTS } from './constants';
import type {
  ValidateBetRequest,
  ValidateBetResponse,
  PlaceBetResponse,
  BetHistoryEntry,
  CreateUserRequest,
  LoginRequest,
  AuthResponse,
  User,
  Match,
} from '@/types/domain';

// cliente axios preconfigurado con baseURL y timeout
const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10_000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// interceptor para inyectar token de auth si existe
api.interceptors.request.use((config) => {
  if (typeof window !== 'undefined') {
    const token = localStorage.getItem('auth_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
  }
  return config;
});

// Health Check
export async function checkHealth(): Promise<boolean> {
  try {
    const { status } = await api.get(ENDPOINTS.HEALTH_CHECK);
    return status === 200;
  } catch {
    return false;
  }
}

// Active matches snapshot
export async function fetchActiveMatches(): Promise<Match[]> {
  // hacemos una especie de simulacion temporal de 500ms
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve([
        { id: 'match-1', home_team: 'Real Madrid', away_team: 'Barcelona', status: 'InPlay', odds: 1.85 },
        { id: 'match-2', home_team: 'Manchester City', away_team: 'Arsenal', status: 'InPlay', odds: 2.1 },
        { id: 'match-3', home_team: 'Bayern Munich', away_team: 'Dortmund', status: 'NotStarted', odds: 1.5 },
      ]);
    }, 500);
  });
}

// Apuestas
export async function validateBet(data: ValidateBetRequest): Promise<ValidateBetResponse> {
  const { data: result } = await api.post<ValidateBetResponse>(ENDPOINTS.BETS, data);
  return result;
}

export async function placeBet(data: ValidateBetRequest): Promise<PlaceBetResponse> {
  const { data: result } = await api.post<PlaceBetResponse>(ENDPOINTS.BETS, data);
  return result;
}

// historial (data fría, cacheada por tanstack query)
export async function fetchBetHistory(userId: string): Promise<BetHistoryEntry[]> {
  const { data } = await api.get<BetHistoryEntry[]>(`${ENDPOINTS.BETS}/history`, {
    params: { user_id: userId },
  });
  return data;
}

// Auth
export async function registerUser(data: CreateUserRequest): Promise<AuthResponse> {
  const { data: result } = await api.post<AuthResponse>(ENDPOINTS.REGISTER, data);
  return result;
}

export async function loginUser(data: LoginRequest): Promise<AuthResponse> {
  const { data: result } = await api.post<AuthResponse>(ENDPOINTS.LOGIN, data);
  return result;
}

// perfil (data fría, staletime: infinito)
export async function fetchCurrentUser(): Promise<User> {
  const { data } = await api.get<User>('/users/me');
  return data;
}

export default api;