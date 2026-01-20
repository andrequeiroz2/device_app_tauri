// TODO: Implementar chamadas para API local
// Base URL da API
const API_BASE_URL = 'http://localhost:3001/api/auth';

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface RegisterCredentials {
  name: string;
  email: string;
  password: string;
}

export interface AuthResponse {
  success: boolean;
  message?: string;
  user?: {
    id: string;
    name: string;
    email: string;
  };
  token?: string;
}

export const authApi = {
  // TODO: Implementar chamada de login para API
  async login(credentials: LoginCredentials): Promise<AuthResponse> {
    // TODO: Substituir por chamada real à API
    // const response = await fetch(`${API_BASE_URL}/login`, {
    //   method: 'POST',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify(credentials),
    // });
    // return response.json();

    console.log('TODO: Login API call', credentials);
    return { success: true, message: 'Login simulado - implementar API' };
  },

  // TODO: Implementar chamada de registro para API
  async register(credentials: RegisterCredentials): Promise<AuthResponse> {
    // TODO: Substituir por chamada real à API
    // const response = await fetch(`${API_BASE_URL}/register`, {
    //   method: 'POST',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify(credentials),
    // });
    // return response.json();

    console.log('TODO: Register API call', credentials);
    return { success: true, message: 'Registro simulado - implementar API' };
  },

  // TODO: Implementar chamada de recuperação de senha para API
  async forgotPassword(email: string): Promise<AuthResponse> {
    // TODO: Substituir por chamada real à API
    // const response = await fetch(`${API_BASE_URL}/forgot-password`, {
    //   method: 'POST',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: JSON.stringify({ email }),
    // });
    // return response.json();

    console.log('TODO: Forgot Password API call', email);
    return { success: true, message: 'Email de recuperação simulado - implementar API' };
  },
};
