import { AxiosError } from 'axios';
import type { ApiError } from '../types';

/**
 * Standard error type for application errors
 */
export interface AppError {
  message: string;
  statusCode?: number;
  details?: unknown;
}

/**
 * Type guard to check if error is an AxiosError
 */
export function isAxiosError(error: unknown): error is AxiosError {
  return (error as AxiosError).isAxiosError === true;
}

/**
 * Extract error message from various error types
 */
export function getErrorMessage(error: unknown): string {
  if (isAxiosError(error)) {
    // Try to extract API error message
    const apiError = error.response?.data as ApiError | undefined;

    if (apiError) {
      switch (apiError.type) {
        case 'NotFound':
        case 'Unauthorized':
        case 'Forbidden':
        case 'DatabaseError':
        case 'InternalError':
        case 'BadRequest':
        case 'Conflict':
          return apiError.data.message;
        case 'ValidationError':
          return `Validation error: ${apiError.data.fields.map(f => f.message).join(', ')}`;
      }
    }

    // Fallback to generic axios error message
    const dataMessage = error.response?.data && typeof error.response.data === 'object' && 'message' in error.response.data
      ? String(error.response.data.message)
      : undefined;
    return dataMessage || error.message || 'An error occurred';
  }

  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
    return error.message;
  }

  return 'An unexpected error occurred';
}

/**
 * Convert unknown error to AppError
 */
export function toAppError(error: unknown): AppError {
  if (isAxiosError(error)) {
    return {
      message: getErrorMessage(error),
      statusCode: error.response?.status,
      details: error.response?.data,
    };
  }

  if (error instanceof Error) {
    return {
      message: error.message,
      details: error,
    };
  }

  return {
    message: getErrorMessage(error),
    details: error,
  };
}
