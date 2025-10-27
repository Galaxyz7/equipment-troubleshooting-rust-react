import { useState, memo } from 'react';
import { issuesAPI } from '../lib/api';
import type { Issue } from '../types/issues';
import CategoryCombobox from './CategoryCombobox';
import { getErrorMessage } from '../lib/errorUtils';

interface CreateIssueModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (issue: Issue) => void;
}

const CreateIssueModal = memo(function CreateIssueModal({ isOpen, onClose, onCreate }: CreateIssueModalProps) {
  const [name, setName] = useState('');
  const [displayCategory, setDisplayCategory] = useState('');
  const [firstQuestion, setFirstQuestion] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);

  // Real-time validation
  const validate = () => {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Issue name is required';
    } else if (name.length < 3) {
      newErrors.name = 'Name must be at least 3 characters';
    } else if (name.length > 100) {
      newErrors.name = 'Name must be less than 100 characters';
    }

    if (!firstQuestion.trim()) {
      newErrors.firstQuestion = 'First question is required';
    } else if (firstQuestion.length < 10) {
      newErrors.firstQuestion = 'Question must be at least 10 characters';
    } else if (firstQuestion.length > 500) {
      newErrors.firstQuestion = 'Question must be less than 500 characters';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) return;

    setLoading(true);
    try {
      // Auto-generate category ID from name (internal implementation detail)
      const categoryId = name.toLowerCase().replace(/\s+/g, '_').replace(/[^a-z0-9_]/g, '');

      const newIssue = await issuesAPI.create({
        name: name.trim(),
        category: categoryId,
        display_category: displayCategory.trim() || null,
        root_question_text: firstQuestion.trim(),
      });

      onCreate(newIssue);
      resetForm();
      onClose();
    } catch (err: unknown) {
      setErrors({
        submit: getErrorMessage(err) || 'Failed to create issue',
      });
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setName('');
    setDisplayCategory('');
    setFirstQuestion('');
    setErrors({});
  };

  const handleClose = () => {
    if (loading) return; // Prevent closing while loading
    resetForm();
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div
        className="bg-white rounded-2xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="bg-gradient-to-r from-purple-600 to-blue-600 text-white p-6 rounded-t-2xl">
          <h2 className="text-2xl font-bold">Create New Issue</h2>
          <p className="text-purple-100 mt-1">Add a new troubleshooting category</p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-6">
          {/* Issue Name */}
          <div>
            <label htmlFor="issue-name" className="block text-sm font-medium text-gray-700 mb-2">
              Issue Name <span className="text-red-500">*</span>
            </label>
            <input
              id="issue-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={validate}
              className={`w-full px-4 py-3 border rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-colors ${
                errors.name ? 'border-red-500' : 'border-gray-300'
              }`}
              placeholder="e.g., Brush Problems, Motor Issues, Blade Alignment"
              disabled={loading}
              autoFocus
            />
            {errors.name && (
              <p className="text-red-600 text-sm mt-1 flex items-center">
                <span className="mr-1">⚠️</span>
                {errors.name}
              </p>
            )}
          </div>

          {/* Display Category */}
          <CategoryCombobox
            value={displayCategory}
            onChange={setDisplayCategory}
            disabled={loading}
            placeholder="Type to search or create a new category"
            label="Display Category"
            optional={true}
            description="Groups related issues together in the user interface"
          />

          {/* First Question */}
          <div>
            <label htmlFor="first-question" className="block text-sm font-medium text-gray-700 mb-2">
              First Question <span className="text-red-500">*</span>
            </label>
            <textarea
              id="first-question"
              value={firstQuestion}
              onChange={(e) => setFirstQuestion(e.target.value)}
              onBlur={validate}
              rows={3}
              className={`w-full px-4 py-3 border rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-colors resize-none ${
                errors.firstQuestion ? 'border-red-500' : 'border-gray-300'
              }`}
              placeholder="e.g., Is the motor making any noise?"
              disabled={loading}
            />
            {errors.firstQuestion && (
              <p className="text-red-600 text-sm mt-1 flex items-center">
                <span className="mr-1">⚠️</span>
                {errors.firstQuestion}
              </p>
            )}
            <p className="text-gray-500 text-sm mt-1 flex items-center">
              <span className="mr-1">🎯</span>
              This will be the first question users see when starting this troubleshooting flow
            </p>
          </div>

          {/* Submit Error */}
          {errors.submit && (
            <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-red-700 text-sm flex items-center">
                <span className="mr-2 text-lg">❌</span>
                {errors.submit}
              </p>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-3 justify-end pt-4 border-t">
            <button
              type="button"
              onClick={handleClose}
              className="px-6 py-3 rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-50 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
              disabled={loading}
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading || Object.keys(errors).length > 0}
              className="px-6 py-3 rounded-lg bg-gradient-to-r from-purple-600 to-blue-600 text-white hover:from-purple-700 hover:to-blue-700 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-medium shadow-lg hover:shadow-xl flex items-center"
            >
              {loading ? (
                <>
                  <div className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-solid border-white border-r-transparent mr-2"></div>
                  Creating...
                </>
              ) : (
                <>
                  <span className="mr-1">✨</span>
                  Create Issue
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
});

export default CreateIssueModal;
