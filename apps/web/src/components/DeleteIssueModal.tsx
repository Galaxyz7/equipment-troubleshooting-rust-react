import { useState, memo } from 'react';
import type { Issue } from '../types/issues';
import { getErrorMessage } from '../lib/errorUtils';
import { logger } from '../lib/logger';

interface DeleteIssueModalProps {
  issue: Issue | null;
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (category: string, deleteSessions: boolean) => Promise<void>;
}

const DeleteIssueModal = memo(function DeleteIssueModal({ issue, isOpen, onClose, onConfirm }: DeleteIssueModalProps) {
  const [deleteSessions, setDeleteSessions] = useState(false);
  const [deleting, setDeleting] = useState(false);

  const handleConfirm = async () => {
    if (!issue) return;

    setDeleting(true);
    try {
      await onConfirm(issue.category, deleteSessions);
      setDeleteSessions(false);
      onClose();
    } catch (error) {
      logger.error('Failed to delete issue', {
        category: issue.category,
        deleteSessions,
        error: getErrorMessage(error)
      });
    } finally {
      setDeleting(false);
    }
  };

  const handleClose = () => {
    setDeleteSessions(false);
    onClose();
  };

  if (!isOpen || !issue) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div
        className="bg-white rounded-xl shadow-2xl max-w-md w-full"
        role="dialog"
        aria-modal="true"
        aria-labelledby="delete-modal-title"
      >
        {/* Header */}
        <div className="bg-red-600 px-6 py-4 flex items-center justify-between rounded-t-xl">
          <div className="flex items-center">
            <span className="text-2xl mr-3" aria-hidden="true">⚠️</span>
            <h2 id="delete-modal-title" className="text-xl font-bold text-white">Delete Issue</h2>
          </div>
          <button
            onClick={handleClose}
            className="text-white hover:text-red-100 text-2xl leading-none"
            aria-label="Close delete confirmation dialog"
          >
            ×
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          <p className="text-gray-800 mb-4">
            Are you sure you want to delete the issue <span className="font-bold">"{issue.name}"</span>?
          </p>
          <p className="text-gray-700 mb-4">
            This will delete all <span className="font-bold">{Number(issue.question_count)}</span> questions in this category.
          </p>

          {/* Checkbox for deleting sessions */}
          <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
            <label className="flex items-start cursor-pointer">
              <input
                type="checkbox"
                checked={deleteSessions}
                onChange={(e) => setDeleteSessions(e.target.checked)}
                className="mt-1 mr-3 h-4 w-4 text-red-600 focus:ring-red-500 border-gray-300 rounded"
              />
              <div>
                <span className="text-sm font-medium text-gray-900">
                  Also delete all session/analytics data for this issue
                </span>
                <p className="text-xs text-gray-600 mt-1">
                  This will permanently remove all troubleshooting session history and analytics data related to this issue.
                </p>
              </div>
            </label>
          </div>

          <div className="bg-red-50 border-l-4 border-red-400 p-4 mb-4">
            <p className="text-sm text-red-800 font-medium">
              This action cannot be undone.
            </p>
          </div>

          {/* Actions */}
          <div className="flex items-center justify-end space-x-3">
            <button
              onClick={handleClose}
              className="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 font-medium"
              disabled={deleting}
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              disabled={deleting}
              className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed font-medium"
            >
              {deleting ? 'Deleting...' : 'Delete Issue'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
});

export default DeleteIssueModal;
