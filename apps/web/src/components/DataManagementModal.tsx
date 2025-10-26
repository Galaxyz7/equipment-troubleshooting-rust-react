import { useState, useEffect } from 'react';
import { adminAPI, issuesAPI } from '../lib/api';
import type { Issue } from '../types/issues';

interface DataManagementModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export default function DataManagementModal({ isOpen, onClose, onSuccess }: DataManagementModalProps) {
  const [timeRange, setTimeRange] = useState<string>('all_time');
  const [category, setCategory] = useState<string>('all');
  const [status, setStatus] = useState<string>('all');
  const [issues, setIssues] = useState<Issue[]>([]);
  const [previewCount, setPreviewCount] = useState<number | null>(null);
  const [confirmText, setConfirmText] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [loadingPreview, setLoadingPreview] = useState(false);

  // Load issues for category filter
  useEffect(() => {
    if (isOpen) {
      loadIssues();
    }
  }, [isOpen]);

  // Load preview count when filters change
  useEffect(() => {
    if (isOpen) {
      loadPreviewCount();
    }
  }, [isOpen, timeRange, category, status]);

  const loadIssues = async () => {
    try {
      const data = await issuesAPI.list();
      setIssues(data);
    } catch (err) {
      console.error('Error loading issues:', err);
    }
  };

  const loadPreviewCount = async () => {
    setLoadingPreview(true);
    try {
      const params = new URLSearchParams();
      if (timeRange !== 'all_time') params.append('time_range', timeRange);
      if (category !== 'all') params.append('category', category);
      if (status !== 'all') params.append('status', status);

      const result = await adminAPI.getSessionsCount(params);
      setPreviewCount(result.count);
    } catch (err) {
      console.error('Error loading preview count:', err);
      setPreviewCount(null);
    } finally {
      setLoadingPreview(false);
    }
  };

  const handleDelete = async () => {
    if (confirmText !== 'DELETE') {
      setError('Please type DELETE to confirm');
      return;
    }

    if (previewCount === 0) {
      setError('No sessions match the selected filters');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const params = new URLSearchParams();
      if (timeRange !== 'all_time') params.append('time_range', timeRange);
      if (category !== 'all') params.append('category', category);
      if (status !== 'all') params.append('status', status);

      await adminAPI.deleteSessions(params);

      onSuccess();
      onClose();

      // Reset form
      setTimeRange('all_time');
      setCategory('all');
      setStatus('all');
      setConfirmText('');
      setPreviewCount(null);
    } catch (err: any) {
      setError(err.response?.data?.error?.data?.message || 'Failed to delete sessions');
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setConfirmText('');
    setError('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="bg-gradient-to-r from-red-600 to-red-700 px-6 py-4 flex items-center justify-between rounded-t-xl">
          <div className="flex items-center">
            <span className="text-3xl mr-3">🗑️</span>
            <div>
              <h2 className="text-2xl font-bold text-white">Data Management</h2>
              <p className="text-red-100 text-sm">Clear session data based on filters</p>
            </div>
          </div>
          <button
            onClick={handleClose}
            className="text-white hover:text-red-100 text-2xl leading-none"
          >
            ×
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          {/* Warning Banner */}
          <div className="bg-yellow-50 border-l-4 border-yellow-400 p-4 mb-6">
            <div className="flex items-start">
              <span className="text-2xl mr-3">⚠️</span>
              <div>
                <h3 className="text-sm font-medium text-yellow-800">Warning: This action cannot be undone</h3>
                <p className="text-sm text-yellow-700 mt-1">
                  Deleted session data will be permanently removed from the database.
                  Issue definitions (questions and answers) will not be affected.
                </p>
              </div>
            </div>
          </div>

          {/* Filters */}
          <div className="space-y-4 mb-6">
            <h3 className="text-lg font-semibold text-gray-900">Filters</h3>

            {/* Time Range */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Time Range
              </label>
              <select
                value={timeRange}
                onChange={(e) => setTimeRange(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              >
                <option value="all_time">All Time</option>
                <option value="past_month">Past 30 Days</option>
                <option value="past_week">Past 7 Days</option>
                <option value="today">Today</option>
              </select>
            </div>

            {/* Category/Issue */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Issue Category
              </label>
              <select
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              >
                <option value="all">All Issues</option>
                {issues.map((issue) => (
                  <option key={issue.category} value={issue.category}>
                    {issue.name}
                  </option>
                ))}
              </select>
            </div>

            {/* Session Status */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Session Status
              </label>
              <select
                value={status}
                onChange={(e) => setStatus(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              >
                <option value="all">All Statuses</option>
                <option value="completed">Completed</option>
                <option value="abandoned">Abandoned</option>
                <option value="active">Active</option>
              </select>
            </div>
          </div>

          {/* Preview Count */}
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-4 mb-6">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700">Sessions matching filters:</span>
              {loadingPreview ? (
                <span className="text-sm text-gray-500">Loading...</span>
              ) : (
                <span className="text-lg font-bold text-gray-900">
                  {previewCount !== null ? previewCount.toLocaleString() : '—'}
                </span>
              )}
            </div>
          </div>

          {/* Confirmation */}
          <div className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Type <span className="font-mono font-bold text-red-600">DELETE</span> to confirm
            </label>
            <input
              type="text"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
              placeholder="Type DELETE here"
            />
          </div>

          {/* Error Message */}
          {error && (
            <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
              {error}
            </div>
          )}

          {/* Actions */}
          <div className="flex items-center justify-end space-x-3">
            <button
              onClick={handleClose}
              className="px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 font-medium"
            >
              Cancel
            </button>
            <button
              onClick={handleDelete}
              disabled={loading || confirmText !== 'DELETE' || previewCount === 0 || loadingPreview}
              className="px-6 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed font-medium"
            >
              {loading ? 'Deleting...' : `Delete ${previewCount || 0} Sessions`}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
