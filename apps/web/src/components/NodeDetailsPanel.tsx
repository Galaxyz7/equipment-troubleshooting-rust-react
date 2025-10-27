import { memo } from 'react';
import { nodesAPI } from '../lib/api';
import type { Node } from '../types';
import { getErrorMessage } from '../lib/errorUtils';

interface NodeDetailsPanelProps {
  isOpen: boolean;
  selectedNode: Node | undefined;
  editingText: string;
  hasUnsavedChanges: boolean;
  onEditingTextChange: (text: string) => void;
  onSave: () => void;
  onDelete: (nodeId: string) => void;
  onClose: () => void;
  onNodeTypeChange: () => Promise<void>;
  setError: (error: string) => void;
}

export const NodeDetailsPanel = memo(function NodeDetailsPanel({
  isOpen,
  selectedNode,
  editingText,
  hasUnsavedChanges,
  onEditingTextChange,
  onSave,
  onDelete,
  onClose,
  onNodeTypeChange,
  setError,
}: NodeDetailsPanelProps) {
  const handleNodeTypeChange = async (newType: 'Question' | 'Conclusion') => {
    if (!selectedNode || selectedNode.node_type === newType) return;
    try {
      await nodesAPI.update(selectedNode.id, { node_type: newType });
      await onNodeTypeChange();
    } catch (err: unknown) {
      setError(`Failed to change node type: ${getErrorMessage(err)}`);
    }
  };

  return (
    <div
      className={`absolute left-0 top-0 bottom-0 w-[350px] bg-white border-r border-gray-200 shadow-xl overflow-y-auto z-10 transition-transform duration-300 ease-in-out ${
        isOpen ? 'translate-x-0' : '-translate-x-full'
      }`}
    >
      <div className="p-6">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-bold text-gray-800">Edit Node</h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-xl font-bold"
            aria-label="Close node panel"
          >
            ✕
          </button>
        </div>

        {selectedNode ? (
          <div>
            {/* Node Type Selector */}
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Node Type
              </label>
              <div className="flex gap-3">
                <button
                  onClick={() => handleNodeTypeChange('Question')}
                  className={`flex-1 px-4 py-2 rounded-md border-2 font-medium transition-all ${
                    selectedNode.node_type === 'Question'
                      ? 'border-blue-500 bg-blue-100 text-blue-800'
                      : 'border-gray-300 bg-white text-gray-600 hover:border-blue-300'
                  }`}
                >
                  ❓ Question
                </button>
                <button
                  onClick={() => handleNodeTypeChange('Conclusion')}
                  className={`flex-1 px-4 py-2 rounded-md border-2 font-medium transition-all ${
                    selectedNode.node_type === 'Conclusion'
                      ? 'border-green-500 bg-green-100 text-green-800'
                      : 'border-gray-300 bg-white text-gray-600 hover:border-green-300'
                  }`}
                >
                  🎯 Conclusion
                </button>
              </div>
            </div>

            {/* Node Text */}
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                {selectedNode.node_type === 'Question' ? 'Question Text' : 'Conclusion Text'}
              </label>
              <textarea
                value={editingText}
                onChange={(e) => onEditingTextChange(e.target.value)}
                rows={6}
                className="w-full px-3 py-2 border border-gray-300 rounded-md resize-none"
                placeholder={
                  selectedNode.node_type === 'Question'
                    ? 'Enter the question text...'
                    : 'Enter the conclusion text...'
                }
              />
              <p className="text-xs text-gray-500 mt-1">
                {editingText.length} characters
              </p>
            </div>

            {/* Node Metadata */}
            <div className="mb-4 p-3 bg-gray-50 border border-gray-200 rounded-md">
              <div className="text-xs space-y-1">
                <div>
                  <span className="font-medium text-gray-600">ID:</span>{' '}
                  <span className="font-mono text-gray-800">{selectedNode.id.substring(0, 8)}...</span>
                </div>
                <div>
                  <span className="font-medium text-gray-600">Category:</span>{' '}
                  <span className="text-gray-800">{selectedNode.category}</span>
                </div>
                {selectedNode.semantic_id && (
                  <div>
                    <span className="font-medium text-gray-600">Semantic ID:</span>{' '}
                    <span className="font-mono text-gray-800">{selectedNode.semantic_id}</span>
                  </div>
                )}
              </div>
            </div>

            {/* Save Button */}
            {hasUnsavedChanges && (
              <div className="mb-4">
                <button
                  onClick={onSave}
                  className="w-full px-4 py-2 rounded-md bg-gradient-to-r from-green-500 to-green-600 text-white font-medium hover:from-green-600 hover:to-green-700 shadow-md"
                >
                  Save Node Changes
                </button>
              </div>
            )}

            {/* Delete Button */}
            <button
              onClick={() => onDelete(selectedNode.id)}
              className="w-full px-3 py-2 rounded-md bg-red-500 text-white font-medium hover:bg-red-600"
            >
              Delete Node
            </button>
          </div>
        ) : (
          <div className="text-center py-12 text-gray-500">
            <p className="text-sm">No node selected</p>
            <p className="text-xs mt-2">Click a node to edit</p>
          </div>
        )}
      </div>
    </div>
  );
});
