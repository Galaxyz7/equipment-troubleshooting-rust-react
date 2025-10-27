import { useState, useEffect, memo } from 'react';
import { adminAPI } from '../lib/api';
import { AccessibleConfirm } from './AccessibleConfirm';
import { getErrorMessage } from '../lib/errorUtils';
import { logger } from '../lib/logger';

interface CategoryManagementModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: (message: string) => void;
}

const CategoryManagementModal = memo(function CategoryManagementModal({ isOpen, onClose, onSuccess }: CategoryManagementModalProps) {
  const [categories, setCategories] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editingCategory, setEditingCategory] = useState<string | null>(null);
  const [newName, setNewName] = useState('');

  // Accessible confirm dialog state
  const [confirmDialog, setConfirmDialog] = useState<{ isOpen: boolean; title: string; message: string; onConfirm: () => void }>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
  });

  useEffect(() => {
    if (isOpen) {
      loadCategories();
    }
  }, [isOpen]);

  const loadCategories = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await adminAPI.getCategories();
      setCategories(data.categories);
    } catch (err: unknown) {
      setError('Failed to load categories');
      logger.error('Failed to load categories for management modal', { error: getErrorMessage(err) });
    } finally {
      setLoading(false);
    }
  };

  const handleRename = async (oldName: string) => {
    if (!newName.trim()) {
      setError('New category name cannot be empty');
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const result = await adminAPI.renameCategory(oldName, newName.trim());
      onSuccess(`Category "${oldName}" renamed to "${newName}". ${result.updated_count} issue(s) updated.`);
      setEditingCategory(null);
      setNewName('');
      await loadCategories();
    } catch (err: unknown) {
      setError(`Failed to rename category: ${getErrorMessage(err)}`);
      logger.error('Failed to rename category', { oldName, newName: newName.trim(), error: getErrorMessage(err) });
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (name: string) => {
    setConfirmDialog({
      isOpen: true,
      title: 'Delete Category',
      message: `Are you sure you want to delete the "${name}" category?\n\nThis will clear the category from all issues using it (they will become uncategorized).`,
      onConfirm: async () => {
        setLoading(true);
        setError(null);
        try {
          const result = await adminAPI.deleteCategory(name);
          onSuccess(`Category "${name}" deleted. ${result.updated_count} issue(s) cleared.`);
          await loadCategories();
        } catch (err: unknown) {
          setError(`Failed to delete category: ${getErrorMessage(err)}`);
          logger.error('Failed to delete category', { categoryName: name, error: getErrorMessage(err) });
        } finally {
          setLoading(false);
        }
      },
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="bg-gradient-to-r from-purple-600 to-blue-600 text-white p-6">
          <h2 className="text-2xl font-bold">Manage Categories</h2>
          <p className="text-purple-100 mt-1">Rename or delete display categories</p>
        </div>

        {/* Error Display */}
        {error && (
          <div className="mx-6 mt-6 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-700 text-sm flex items-center">
              <span className="mr-2 text-lg">❌</span>
              {error}
            </p>
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {loading && categories.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-purple-600 border-r-transparent mb-4"></div>
              <p>Loading categories...</p>
            </div>
          ) : categories.length === 0 ? (
            <div className="text-center py-12 text-gray-500">
              <p className="text-lg mb-2">No categories found</p>
              <p className="text-sm">Create categories by adding them when creating or editing issues.</p>
            </div>
          ) : (
            <div className="space-y-3">
              {categories.map((category) => (
                <div key={category} className="border border-gray-200 rounded-lg p-4">
                  {editingCategory === category ? (
                    // Edit mode
                    <div className="space-y-3">
                      <input
                        type="text"
                        value={newName}
                        onChange={(e) => setNewName(e.target.value)}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500"
                        placeholder="Enter new category name"
                        autoFocus
                        disabled={loading}
                      />
                      <div className="flex gap-2">
                        <button
                          onClick={() => handleRename(category)}
                          disabled={loading || !newName.trim()}
                          className="flex-1 px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                        >
                          {loading ? 'Saving...' : 'Save'}
                        </button>
                        <button
                          onClick={() => {
                            setEditingCategory(null);
                            setNewName('');
                          }}
                          disabled={loading}
                          className="flex-1 px-4 py-2 bg-gray-300 text-gray-700 rounded-md hover:bg-gray-400 disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  ) : (
                    // View mode
                    <div className="flex items-center justify-between">
                      <div className="flex items-center">
                        <span className="inline-block w-3 h-3 bg-purple-500 rounded-full mr-3"></span>
                        <span className="text-lg font-medium text-gray-800">{category}</span>
                      </div>
                      <div className="flex gap-2">
                        <button
                          onClick={() => {
                            setEditingCategory(category);
                            setNewName(category);
                            setError(null);
                          }}
                          disabled={loading}
                          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed font-medium text-sm"
                        >
                          Rename
                        </button>
                        <button
                          onClick={() => handleDelete(category)}
                          disabled={loading}
                          className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed font-medium text-sm"
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="border-t border-gray-200 p-6">
          <button
            onClick={onClose}
            disabled={loading}
            className="w-full px-6 py-3 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 disabled:opacity-50 disabled:cursor-not-allowed font-medium"
            aria-label="Close category management dialog"
          >
            Close
          </button>
        </div>
      </div>

      {/* Accessible confirm dialog (replaces confirm()) */}
      <AccessibleConfirm
        isOpen={confirmDialog.isOpen}
        onClose={() => setConfirmDialog({ ...confirmDialog, isOpen: false })}
        onConfirm={confirmDialog.onConfirm}
        title={confirmDialog.title}
        message={confirmDialog.message}
        variant="danger"
      />
    </div>
  );
});

export default CategoryManagementModal;
