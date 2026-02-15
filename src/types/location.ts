export type Location = {
  id: number;
  uuid: string;
  user_id: number;
  name: string;
  description?: string | null;
  address: string;
  is_active: boolean;
  image_path?: string | null;
  thumb_path?: string | null;
  image_original_name?: string | null;
  image_mime?: string | null;
  image_size_bytes?: number | null;
  image_checksum_sha256?: string | null;
  created_at: string;
  updated_at: string;
};

export type LocationPublic = {
  uuid: string;
  name: string;
  description?: string | null;
  address: string;
  is_active: boolean;
  image_path?: string | null;
  thumb_path?: string | null;
  image_original_name?: string | null;
  image_mime?: string | null;
  image_size_bytes?: number | null;
  image_checksum_sha256?: string | null;
  created_at: string;
  updated_at: string;
};

export type LocationCreateInput = {
  name: string;
  address: string;
  description?: string;
};

export type LocationImageInput = {
  data_base64: string;
  original_name: string;
  mime: string;
  size_bytes: number;
};

export type LocationCreateCommandInput = {
  location: LocationCreateInput;
  image?: LocationImageInput;
};

export type LocationListResponse = {
  items: LocationPublic[];
  total: number;
  page: number;
  page_size: number;
};

