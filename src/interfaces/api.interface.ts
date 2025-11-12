export interface InvokeResponse<T> {
  data: T;
  message: string;
}

export interface SuccessResponse<T> {
  data: T;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
}
