import { memo } from 'react';
import type { Connection as APIConnection, IssueGraph } from '../types';

interface ConnectionDetailsPanelProps {
  isOpen: boolean;
  selectedConnection: APIConnection | undefined;
  graphData: IssueGraph | null;
  editingConnections: Record<string, { label: string; to_node_id: string }>;
  hasUnsavedChanges: boolean;
  onConnectionChange: (connectionId: string, label: string, toNodeId: string) => void;
  onSave: () => void;
  onDelete: (connectionId: string) => void;
  onClose: () => void;
}

export const ConnectionDetailsPanel = memo(function ConnectionDetailsPanel({
  isOpen,
  selectedConnection,
  graphData,
  editingConnections,
  hasUnsavedChanges,
  onConnectionChange,
  onSave,
  onDelete,
  onClose,
}: ConnectionDetailsPanelProps) {
  if (!selectedConnection) {
    return (
      <div
        className={`absolute right-0 top-0 bottom-0 w-[350px] bg-white border-l border-gray-200 shadow-xl overflow-y-auto z-10 transition-transform duration-300 ease-in-out ${
          isOpen ? 'translate-x-0' : 'translate-x-full'
        }`}
      >
        <div className="p-6">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-bold text-gray-800">Edit Connection</h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 text-xl font-bold"
              aria-label="Close connection panel"
            >
              ✕
            </button>
          </div>
          <div className="text-center py-12 text-gray-500">
            <p className="text-sm">No connection selected</p>
            <p className="text-xs mt-2">Click a connection to edit</p>
          </div>
        </div>
      </div>
    );
  }

  const fromNode = graphData?.nodes.find(n => n.id === selectedConnection.from_node_id);
  const toNode = graphData?.nodes.find(n => n.id === selectedConnection.to_node_id);
  const currentLabel = editingConnections[selectedConnection.id]?.label ?? selectedConnection.label;
  const currentToNodeId = editingConnections[selectedConnection.id]?.to_node_id ?? selectedConnection.to_node_id;

  return (
    <div
      className={`absolute right-0 top-0 bottom-0 w-[350px] bg-white border-l border-gray-200 shadow-xl overflow-y-auto z-10 transition-transform duration-300 ease-in-out ${
        isOpen ? 'translate-x-0' : 'translate-x-full'
      }`}
    >
      <div className="p-6">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-bold text-gray-800">Edit Connection</h3>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-xl font-bold"
            aria-label="Close connection panel"
          >
            ✕
          </button>
        </div>

        <div>
          {/* Connection Info */}
          <div className="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-md">
            <p className="text-xs text-blue-600 font-medium mb-1">From:</p>
            <p className="text-sm text-gray-800 mb-2">
              {fromNode?.text.substring(0, 50)}{fromNode && fromNode.text.length > 50 ? '...' : ''}
            </p>
            <p className="text-xs text-blue-600 font-medium mb-1">To:</p>
            <p className="text-sm text-gray-800">
              {toNode?.text.substring(0, 50)}{toNode && toNode.text.length > 50 ? '...' : ''}
            </p>
          </div>

          {/* Connection Label */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Answer / Option Label
            </label>
            <input
              type="text"
              value={currentLabel}
              onChange={(e) => onConnectionChange(selectedConnection.id, e.target.value, currentToNodeId)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
              placeholder="e.g., Yes, No, Worn..."
            />
            <p className="text-xs text-gray-500 mt-1">
              This label appears as an answer option for users
            </p>
          </div>

          {/* Target Node Selector */}
          <div className="mb-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Target Node
            </label>
            <select
              value={currentToNodeId}
              onChange={(e) => onConnectionChange(selectedConnection.id, currentLabel, e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md"
            >
              {graphData?.nodes
                .filter(n => n.id !== selectedConnection.from_node_id)
                .map(n => (
                  <option key={n.id} value={n.id}>
                    {n.node_type === 'Conclusion' ? '🎯 ' : '❓ '}
                    {n.text.substring(0, 50)}{n.text.length > 50 ? '...' : ''}
                  </option>
                ))
              }
            </select>
            <p className="text-xs text-gray-500 mt-1">
              Where this answer leads to
            </p>
          </div>

          {/* Connection Metadata */}
          <div className="mb-4 p-3 bg-gray-50 border border-gray-200 rounded-md">
            <div className="text-xs space-y-1">
              <div>
                <span className="font-medium text-gray-600">Connection ID:</span>{' '}
                <span className="font-mono text-gray-800">{selectedConnection.id.substring(0, 8)}...</span>
              </div>
              <div>
                <span className="font-medium text-gray-600">Order:</span>{' '}
                <span className="text-gray-800">{selectedConnection.order_index}</span>
              </div>
            </div>
          </div>

          {/* Save Connection Button */}
          {hasUnsavedChanges && (
            <div className="mb-4">
              <button
                onClick={onSave}
                className="w-full px-4 py-2 rounded-md bg-gradient-to-r from-green-500 to-green-600 text-white font-medium hover:from-green-600 hover:to-green-700 shadow-md"
              >
                Save Connection Changes
              </button>
            </div>
          )}

          {/* Delete Connection Button */}
          <button
            onClick={() => onDelete(selectedConnection.id)}
            className="w-full px-3 py-2 rounded-md bg-red-500 text-white font-medium hover:bg-red-600"
          >
            Delete Connection
          </button>
        </div>
      </div>
    </div>
  );
});
