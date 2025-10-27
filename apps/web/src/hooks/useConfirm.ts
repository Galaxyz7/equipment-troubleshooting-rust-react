import { useState, useCallback } from 'react';

/**
 * Confirmation dialog configuration
 */
export interface ConfirmConfig {
  title: string;
  message: string;
  variant?: 'default' | 'danger';
  onConfirm: () => void | Promise<void>;
}

/**
 * Custom hook for managing confirmation dialogs
 *
 * Provides a consistent interface for showing confirmation dialogs across
 * the application. Replaces inline confirm dialog state management.
 *
 * @example
 * ```tsx
 * const { confirmState, showConfirm, hideConfirm, handleConfirm } = useConfirm();
 *
 * // Show danger confirmation
 * const handleDelete = () => {
 *   showConfirm({
 *     title: 'Delete Item',
 *     message: 'Are you sure? This cannot be undone.',
 *     variant: 'danger',
 *     onConfirm: async () => {
 *       await api.delete('/items/123');
 *       console.log('Deleted!');
 *     }
 *   });
 * };
 *
 * // In JSX
 * <AccessibleConfirm
 *   isOpen={confirmState.isOpen}
 *   title={confirmState.title}
 *   message={confirmState.message}
 *   variant={confirmState.variant}
 *   onConfirm={handleConfirm}
 *   onCancel={hideConfirm}
 * />
 * ```
 */
export function useConfirm() {
  const [confirmState, setConfirmState] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    variant: 'default' | 'danger';
    onConfirm: () => void | Promise<void>;
  }>({
    isOpen: false,
    title: '',
    message: '',
    variant: 'default',
    onConfirm: () => {},
  });

  const showConfirm = useCallback((config: ConfirmConfig) => {
    setConfirmState({
      isOpen: true,
      title: config.title,
      message: config.message,
      variant: config.variant || 'default',
      onConfirm: config.onConfirm,
    });
  }, []);

  const hideConfirm = useCallback(() => {
    setConfirmState((prev) => ({
      ...prev,
      isOpen: false,
    }));
  }, []);

  const handleConfirm = useCallback(
    async () => {
      await confirmState.onConfirm();
      hideConfirm();
    },
    // ESLint wants us to include the entire confirmState object, but we only need
    // confirmState.onConfirm which is already in the dependency array
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [confirmState.onConfirm, hideConfirm]
  );

  return {
    confirmState,
    showConfirm,
    hideConfirm,
    handleConfirm,
  };
}
