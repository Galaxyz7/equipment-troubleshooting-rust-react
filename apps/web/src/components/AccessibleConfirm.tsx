import { AccessibleDialog } from './AccessibleDialog';

interface AccessibleConfirmProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: 'default' | 'danger';
}

/**
 * Accessible confirmation dialog to replace window.confirm()
 * Provides proper ARIA attributes, keyboard navigation, and clear action buttons
 */
export function AccessibleConfirm({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  variant = 'default',
}: AccessibleConfirmProps) {
  const handleConfirm = () => {
    onConfirm();
    onClose();
  };

  const confirmButtonClass =
    variant === 'danger'
      ? 'bg-red-600 hover:bg-red-700 text-white'
      : 'bg-gradient-to-r from-[#667eea] to-[#764ba2] hover:opacity-90 text-white';

  return (
    <AccessibleDialog
      isOpen={isOpen}
      onClose={onClose}
      title={title}
      initialFocus="content"
    >
      <div className="mb-6">
        <p className="text-gray-700 whitespace-pre-wrap">{message}</p>
      </div>
      <div className="flex justify-end space-x-3">
        <button
          onClick={onClose}
          className="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300 transition-colors font-medium"
          aria-label={`${cancelText} and close dialog`}
        >
          {cancelText}
        </button>
        <button
          onClick={handleConfirm}
          className={`px-4 py-2 rounded-md transition-all font-medium ${confirmButtonClass}`}
          aria-label={`${confirmText} this action`}
          autoFocus
        >
          {confirmText}
        </button>
      </div>
    </AccessibleDialog>
  );
}
