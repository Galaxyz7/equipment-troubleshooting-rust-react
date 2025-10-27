import { useState, useCallback } from 'react';

/**
 * Toast/Alert notification type
 */
export type ToastType = 'success' | 'info' | 'error' | 'warning';

/**
 * Toast message configuration
 */
export interface ToastConfig {
  title: string;
  message: string;
  type?: ToastType;
}

/**
 * Custom hook for managing toast/alert notifications
 *
 * Provides a consistent interface for displaying success, error, and info messages
 * across the application. Replaces inline alert state management.
 *
 * @example
 * ```tsx
 * const { showToast, toastState, hideToast } = useToast();
 *
 * // Show success toast
 * showToast({ title: 'Success', message: 'Operation completed', type: 'success' });
 *
 * // Show error toast
 * showToast({ title: 'Error', message: 'Something went wrong', type: 'error' });
 *
 * // In JSX
 * <AccessibleAlert
 *   isOpen={toastState.isOpen}
 *   title={toastState.title}
 *   message={toastState.message}
 *   type={toastState.type}
 *   onClose={hideToast}
 * />
 * ```
 */
export function useToast() {
  const [toastState, setToastState] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    type: ToastType;
  }>({
    isOpen: false,
    title: '',
    message: '',
    type: 'info',
  });

  const showToast = useCallback(({ title, message, type = 'info' }: ToastConfig) => {
    setToastState({
      isOpen: true,
      title,
      message,
      type,
    });
  }, []);

  const hideToast = useCallback(() => {
    setToastState((prev) => ({
      ...prev,
      isOpen: false,
    }));
  }, []);

  return {
    toastState,
    showToast,
    hideToast,
  };
}
