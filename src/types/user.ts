export type CreateUserResponse = {
  uuid: string;
  username: string;
  email: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type UserResponse = {
  uuid: string;
  username: string;
  email: string;
  is_active?: boolean;
  created_at?: string;
  updated_at?: string;
};

