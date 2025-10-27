import { describe, it, expect } from 'vitest';
import { getErrorMessage, toAppError, isAxiosError } from './errorUtils';
import type { AxiosError } from 'axios';

describe('errorUtils', () => {
  describe('isAxiosError', () => {
    it('should return true for Axios errors', () => {
      const axiosError = {
        isAxiosError: true,
        message: 'Network error',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      expect(isAxiosError(axiosError)).toBe(true);
    });

    it('should return false for non-Axios errors', () => {
      const regularError = new Error('Regular error');
      expect(isAxiosError(regularError)).toBe(false);
    });

    it('should return false for unknown types', () => {
      expect(isAxiosError('string error')).toBe(false);
      expect(isAxiosError(null)).toBe(false);
      expect(isAxiosError(undefined)).toBe(false);
      expect(isAxiosError({})).toBe(false);
    });
  });

  describe('getErrorMessage', () => {
    it('should extract message from Axios error with NotFound type', () => {
      const axiosError = {
        isAxiosError: true,
        response: {
          data: {
            type: 'NotFound',
            data: { message: 'Resource not found' },
          },
        },
        message: 'Request failed',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      expect(getErrorMessage(axiosError)).toBe('Resource not found');
    });

    it('should extract message from Axios error with Unauthorized type', () => {
      const axiosError = {
        isAxiosError: true,
        response: {
          data: {
            type: 'Unauthorized',
            data: { message: 'Invalid credentials' },
          },
        },
        message: 'Request failed',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      expect(getErrorMessage(axiosError)).toBe('Invalid credentials');
    });

    it('should extract validation errors and format them', () => {
      const axiosError = {
        isAxiosError: true,
        response: {
          data: {
            type: 'ValidationError',
            data: {
              fields: [
                { field: 'email', message: 'Invalid email format' },
                { field: 'password', message: 'Password too short' },
              ],
            },
          },
        },
        message: 'Validation failed',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      expect(getErrorMessage(axiosError)).toBe(
        'Validation error: Invalid email format, Password too short'
      );
    });

    it('should fall back to response data message if no type', () => {
      const axiosError = {
        isAxiosError: true,
        response: {
          data: {
            message: 'Server error occurred',
          },
        },
        message: 'Request failed',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      const message = getErrorMessage(axiosError);
      expect(message).toContain('Server error occurred');
    });

    it('should fall back to axios message if no response data', () => {
      const axiosError = {
        isAxiosError: true,
        message: 'Network Error',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      expect(getErrorMessage(axiosError)).toBe('Network Error');
    });

    it('should handle regular Error objects', () => {
      const error = new Error('Something went wrong');
      expect(getErrorMessage(error)).toBe('Something went wrong');
    });

    it('should handle string errors', () => {
      expect(getErrorMessage('Simple error string')).toBe('Simple error string');
    });

    it('should handle unknown error types', () => {
      expect(getErrorMessage(null)).toBe('An unexpected error occurred');
      expect(getErrorMessage(undefined)).toBe('An unexpected error occurred');
      expect(getErrorMessage(123)).toBe('An unexpected error occurred');
    });
  });

  describe('toAppError', () => {
    it('should convert Axios error with status code', () => {
      const responseData = {
        type: 'NotFound',
        data: { message: 'User not found' },
      };
      const axiosError = {
        isAxiosError: true,
        response: {
          status: 404,
          data: responseData,
        },
        message: 'Request failed',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      const appError = toAppError(axiosError);
      expect(appError.message).toBe('User not found');
      expect(appError.statusCode).toBe(404);
      expect(appError.details).toBe(responseData);
    });

    it('should convert Axios error without status code', () => {
      const axiosError = {
        isAxiosError: true,
        message: 'Network Error',
        name: 'AxiosError',
        toJSON: () => ({}),
      } as AxiosError;

      const appError = toAppError(axiosError);
      expect(appError.message).toBe('Network Error');
      expect(appError.statusCode).toBeUndefined();
      expect(appError.details).toBeUndefined();
    });

    it('should convert regular Error objects', () => {
      const error = new Error('Database connection failed');
      const appError = toAppError(error);

      expect(appError.message).toBe('Database connection failed');
      expect(appError.statusCode).toBeUndefined();
      expect(appError.details).toBe(error);
    });

    it('should convert string errors', () => {
      const appError = toAppError('Operation failed');

      expect(appError.message).toBe('Operation failed');
      expect(appError.statusCode).toBeUndefined();
      expect(appError.details).toBe('Operation failed');
    });

    it('should convert unknown errors', () => {
      const appError = toAppError({ code: 'UNKNOWN' });

      expect(appError.message).toBe('An unexpected error occurred');
      expect(appError.statusCode).toBeUndefined();
      expect(appError.details).toEqual({ code: 'UNKNOWN' });
    });
  });
});
