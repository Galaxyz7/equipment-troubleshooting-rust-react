import { memo } from 'react';
import CategoryCombobox from './CategoryCombobox';

interface IssueMetadataHeaderProps {
  category: string;
  editingIssueName: string;
  editingDisplayCategory: string;
  hasUnsavedChanges: boolean;
  nodesCount: number;
  connectionsCount: number;
  loading: boolean;
  onIssueNameChange: (name: string) => void;
  onDisplayCategoryChange: (category: string) => void;
  onSaveMetadata: () => void;
  onCreateNode: () => void;
  onSaveLayout: () => void;
  onClose: () => void;
  hasLayoutChanges: boolean;
}

export const IssueMetadataHeader = memo(function IssueMetadataHeader({
  category,
  editingIssueName,
  editingDisplayCategory,
  hasUnsavedChanges,
  nodesCount,
  connectionsCount,
  loading,
  onIssueNameChange,
  onDisplayCategoryChange,
  onSaveMetadata,
  onCreateNode,
  onSaveLayout,
  onClose,
  hasLayoutChanges,
}: IssueMetadataHeaderProps) {
  return (
    <div className="bg-white p-4 shadow-lg">
      <div className="flex justify-between items-start mb-3">
        <div className="flex-1">
          <h2 className="text-2xl font-bold text-gray-800 mb-2">Edit Decision Tree</h2>
          <div className="flex gap-4 items-start">
            <div className="flex-1">
              <label className="block text-xs font-medium text-gray-600 mb-1">Issue Name</label>
              <input
                type="text"
                value={editingIssueName}
                onChange={(e) => onIssueNameChange(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                placeholder="e.g., Brush Problems"
              />
            </div>
            <div className="flex-1">
              <CategoryCombobox
                value={editingDisplayCategory}
                onChange={onDisplayCategoryChange}
                placeholder="Type to search or create"
                className="text-sm"
              />
              <label className="block text-xs font-medium text-gray-600 mt-1">Display Category (optional)</label>
            </div>
          </div>
          <p className="text-xs text-gray-500 mt-2">
            Category ID: <span className="font-mono bg-gray-100 px-2 py-1 rounded">{category}</span> •{' '}
            {nodesCount} nodes, {connectionsCount} connections
          </p>
        </div>
        <div className="flex gap-3 ml-4">
          {hasUnsavedChanges && (
            <button
              onClick={onSaveMetadata}
              disabled={loading}
              className="px-4 py-2 rounded-md bg-gradient-to-r from-green-500 to-green-600 text-white font-medium hover:from-green-600 hover:to-green-700 shadow-md text-sm whitespace-nowrap"
              aria-label="Save issue metadata"
            >
              💾 Save Metadata
            </button>
          )}
          <button
            onClick={onCreateNode}
            className="px-4 py-2 rounded-md bg-green-500 text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium text-sm"
            aria-label="Create new node"
          >
            ➕ New Node
          </button>
          <button
            onClick={onSaveLayout}
            disabled={loading}
            className={`px-6 py-2 rounded-md text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium ${
              hasLayoutChanges
                ? 'bg-gradient-to-br from-[#667eea] to-[#764ba2]'
                : 'bg-gray-400 cursor-not-allowed'
            }`}
            aria-label="Save graph layout"
          >
            {loading ? '💾 Saving...' : hasLayoutChanges ? '💾 Save Layout *' : '💾 No Changes'}
          </button>
          <button
            onClick={onClose}
            className="px-6 py-2 rounded-md bg-gray-200 text-gray-700 border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
            aria-label="Close tree editor"
          >
            ✕ Close
          </button>
        </div>
      </div>
    </div>
  );
});
