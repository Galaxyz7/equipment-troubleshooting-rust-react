import { useState, useEffect, useCallback } from 'react';
import ReactFlow, {
  Node as FlowNode,
  Edge,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  Connection as FlowConnection,
  MarkerType,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { issuesAPI, nodesAPI, connectionsAPI } from '../lib/api';
import type { IssueGraph, UpdateNode, UpdateConnection } from '../types';

interface TreeEditorModalProps {
  category: string;
  issueName: string;
  onClose: () => void;
  onSave: () => void;
}

export default function TreeEditorModal({ category, issueName, onClose, onSave }: TreeEditorModalProps) {
  const [flowNodes, setFlowNodes, onFlowNodesChange] = useNodesState([]);
  const [flowEdges, setFlowEdges, onFlowEdgesChange] = useEdgesState([]);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [selectedConnectionId, setSelectedConnectionId] = useState<string | null>(null);
  const [openPanel, setOpenPanel] = useState<'none' | 'node' | 'connection'>('none');
  const [graphData, setGraphData] = useState<IssueGraph | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // Local state for editing (to avoid auto-save on every keystroke)
  const [editingText, setEditingText] = useState<string>('');
  const [editingSemanticId, setEditingSemanticId] = useState<string>('');
  const [hasUnsavedNodeChanges, setHasUnsavedNodeChanges] = useState(false);

  // Local state for connection editing (track changes by connection ID)
  const [editingConnections, setEditingConnections] = useState<Record<string, { label: string; to_node_id: string }>>({});

  // Local state for issue metadata editing
  const [editingIssueName, setEditingIssueName] = useState<string>(issueName);
  const [editingDisplayCategory, setEditingDisplayCategory] = useState<string>('');
  const [hasUnsavedIssueChanges, setHasUnsavedIssueChanges] = useState(false);
  const [issueData, setIssueData] = useState<any>(null);

  // Get currently selected node
  const selectedNode = graphData?.nodes.find(n => n.id === selectedNodeId);

  // Get currently selected connection
  const selectedConnection = graphData?.connections.find(c => c.id === selectedConnectionId);

  // Initialize issue editing state when issue data changes
  useEffect(() => {
    if (issueData) {
      setEditingIssueName(issueData.name || issueName);
      setEditingDisplayCategory(issueData.display_category || '');
      setHasUnsavedIssueChanges(false);
    }
  }, [issueData, issueName]);

  // Initialize local editing state when selected node changes
  useEffect(() => {
    if (selectedNode) {
      setEditingText(selectedNode.text);
      setEditingSemanticId(selectedNode.semantic_id || '');
      setHasUnsavedNodeChanges(false);
    }
  }, [selectedNode]);

  // Initialize connection editing state when selected connection changes
  useEffect(() => {
    if (selectedConnection) {
      setEditingConnections(prev => ({
        ...prev,
        [selectedConnection.id]: {
          label: selectedConnection.label,
          to_node_id: selectedConnection.to_node_id,
        },
      }));
      setHasUnsavedNodeChanges(false);
    }
  }, [selectedConnection]);

  const convertGraphToFlow = useCallback((graph: IssueGraph) => {
    const reactFlowNodes: FlowNode[] = [];
    const reactFlowEdges: Edge[] = [];

    // Load saved layout positions
    const layoutKey = `graph_layout_${category}`;
    const savedPositions = localStorage.getItem(layoutKey);
    const nodePositions: Record<string, { x: number; y: number }> = savedPositions
      ? JSON.parse(savedPositions)
      : {};

    // Create React Flow nodes from graph nodes
    graph.nodes.forEach((node, index) => {
      const savedPos = nodePositions[node.id] || (node.position_x !== null && node.position_y !== null)
        ? { x: node.position_x, y: node.position_y }
        : null;

      const x = savedPos?.x ?? (index % 3) * 350;
      const y = savedPos?.y ?? Math.floor(index / 3) * 200;

      reactFlowNodes.push({
        id: node.id,
        type: node.node_type === 'Conclusion' ? 'output' : 'default',
        position: { x, y },
        data: {
          label: (
            <div className="p-2">
              <div className="font-semibold text-sm">
                {node.node_type === 'Conclusion' ? '🎯 ' : '❓ '}
                {node.text.length > 60 ? node.text.substring(0, 60) + '...' : node.text}
              </div>
            </div>
          )
        },
        style: {
          background: node.node_type === 'Conclusion' ? '#dcfce7' : '#fff',
          border: '2px solid ' + (node.node_type === 'Conclusion' ? '#16a34a' : '#667eea'),
          borderRadius: '8px',
          width: 250,
        },
      });
    });

    // Create React Flow edges from connections
    graph.connections.forEach(connection => {
      reactFlowEdges.push({
        id: connection.id,
        source: connection.from_node_id,
        target: connection.to_node_id,
        label: connection.label,
        type: 'smoothstep',
        animated: true,
        markerEnd: {
          type: MarkerType.ArrowClosed,
        },
      });
    });

    setFlowNodes(reactFlowNodes);
    setFlowEdges(reactFlowEdges);
  }, [category, setFlowNodes, setFlowEdges]);

  const loadGraph = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await issuesAPI.getGraph(category);
      setGraphData(data);
      convertGraphToFlow(data);
    } catch (err: any) {
      setError(`Failed to load graph: ${err.message}`);
      console.error('Error loading graph:', err);
    } finally {
      setLoading(false);
    }
  }, [category, convertGraphToFlow]);

  const loadIssueData = useCallback(async () => {
    try {
      const issues = await issuesAPI.list();
      const issue = issues.find(i => i.category === category);
      if (issue) {
        setIssueData(issue);
      }
    } catch (err: any) {
      console.error('Error loading issue data:', err);
    }
  }, [category]);

  // Load graph data from API
  useEffect(() => {
    loadGraph();
    loadIssueData();
  }, [loadGraph, loadIssueData]);

  // Warn user about unsaved changes when trying to close browser/tab
  useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (hasChanges || hasUnsavedNodeChanges || hasUnsavedIssueChanges) {
        e.preventDefault();
        // Modern browsers ignore custom messages and show a standard one
        e.returnValue = '';
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, [hasChanges, hasUnsavedNodeChanges, hasUnsavedIssueChanges]);

  const handleSaveIssue = async () => {
    if (!hasUnsavedIssueChanges) return;

    try {
      setLoading(true);
      setError(null);

      const updateData: any = {};
      if (editingIssueName !== (issueData?.name || issueName)) {
        updateData.name = editingIssueName;
      }
      if (editingDisplayCategory !== (issueData?.display_category || '')) {
        updateData.display_category = editingDisplayCategory || null;
      }

      if (Object.keys(updateData).length > 0) {
        const updatedIssue = await issuesAPI.update(category, updateData);
        setIssueData(updatedIssue);
        setHasUnsavedIssueChanges(false);
        alert('Issue metadata saved successfully!');
      }
    } catch (err: any) {
      setError(`Failed to save issue metadata: ${err.message}`);
      console.error('Error saving issue:', err);
    } finally {
      setLoading(false);
    }
  };

  // Track node position changes
  const handleNodesChange = useCallback((changes: any) => {
    onFlowNodesChange(changes);
    const hasPositionChange = changes.some((change: any) =>
      change.type === 'position' && change.dragging === false
    );
    if (hasPositionChange) {
      setHasChanges(true);
    }
  }, [onFlowNodesChange]);

  // Handle drag-drop connection creation
  const onConnect = useCallback(
    async (params: FlowConnection) => {
      if (!graphData || !params.source || !params.target) return;

      // Prompt for connection label
      const label = prompt('Enter connection label (e.g., "Yes", "No"):');
      if (!label || label.trim() === '') return;

      try {
        // Count existing connections from source node for order_index
        const existingConnections = graphData.connections.filter(c => c.from_node_id === params.source);

        await connectionsAPI.create({
          from_node_id: params.source,
          to_node_id: params.target,
          label: label.trim(),
          order_index: existingConnections.length,
        });

        await loadGraph();
        setHasChanges(false);
      } catch (err: any) {
        setError(`Failed to create connection: ${err.message}`);
        console.error('Error creating connection:', err);
      }
    },
    [graphData, loadGraph]
  );

  const onNodeClick = useCallback((_event: React.MouseEvent, node: FlowNode) => {
    setSelectedNodeId(node.id);
    setSelectedConnectionId(null);
    setOpenPanel('node');
  }, []);

  const onEdgeClick = useCallback((_event: React.MouseEvent, edge: Edge) => {
    setSelectedConnectionId(edge.id);
    setSelectedNodeId(null);
    setOpenPanel('connection');
  }, []);

  const onPaneClick = useCallback(() => {
    // React Flow only fires this when clicking (not dragging) the pane
    setSelectedNodeId(null);
    setSelectedConnectionId(null);
    setOpenPanel('none');
  }, []);

  // Save node changes from local state to API
  const handleSaveNode = async () => {
    if (!selectedNode || !hasUnsavedNodeChanges) return;

    try {
      const nodeUpdates: UpdateNode = {};

      if (editingText !== selectedNode.text) {
        nodeUpdates.text = editingText;
      }
      if (editingSemanticId !== (selectedNode.semantic_id || '')) {
        nodeUpdates.semantic_id = editingSemanticId || undefined;
      }

      if (Object.keys(nodeUpdates).length > 0) {
        await nodesAPI.update(selectedNode.id, nodeUpdates);
        await loadGraph();
        setHasUnsavedNodeChanges(false);
        setHasChanges(false);
      }
    } catch (err: any) {
      setError(`Failed to update node: ${err.message}`);
      console.error('Error updating node:', err);
    }
  };

  // Save connection changes from local state to API
  const handleSaveConnection = async () => {
    if (!selectedConnection || !hasUnsavedNodeChanges) return;

    try {
      const editedConn = editingConnections[selectedConnection.id];
      if (!editedConn) return;

      const connUpdates: UpdateConnection = {};

      if (editedConn.label !== selectedConnection.label) {
        connUpdates.label = editedConn.label;
      }
      if (editedConn.to_node_id !== selectedConnection.to_node_id) {
        connUpdates.to_node_id = editedConn.to_node_id;
      }

      if (Object.keys(connUpdates).length > 0) {
        await connectionsAPI.update(selectedConnection.id, connUpdates);
        await loadGraph();
        setHasUnsavedNodeChanges(false);
        setHasChanges(false);
      }
    } catch (err: any) {
      setError(`Failed to update connection: ${err.message}`);
      console.error('Error updating connection:', err);
    }
  };

  // Delete connection
  const handleDeleteConnection = async (connId: string) => {
    if (!confirm('Delete this connection?')) return;

    try {
      await connectionsAPI.delete(connId);
      await loadGraph();
      setSelectedConnectionId(null);
      setOpenPanel('none');
      setHasChanges(false);
    } catch (err: any) {
      setError(`Failed to delete connection: ${err.message}`);
      console.error('Error deleting connection:', err);
    }
  };

  // Delete node
  const handleDeleteNode = async (nodeId: string) => {
    const node = graphData?.nodes.find(n => n.id === nodeId);
    if (!node) return;

    if (!confirm(`Delete node "${node.text}"? This will also delete all connections.`)) {
      return;
    }

    try {
      await nodesAPI.delete(nodeId);
      await loadGraph();
      setSelectedNodeId(null);
      setOpenPanel('none');
      setHasChanges(false);
    } catch (err: any) {
      setError(`Failed to delete node: ${err.message}`);
      console.error('Error deleting node:', err);
    }
  };

  // Create new node
  const handleCreateNode = async () => {
    const nodeType = confirm('Create a Question node? (Cancel for Conclusion)')
      ? 'Question'
      : 'Conclusion';

    const text = prompt(`Enter ${nodeType} text:`);
    if (!text || text.trim() === '') return;

    // Auto-generate semantic_id from text (hidden from user)
    const semanticId = text
      .toLowerCase()
      .replace(/[^a-z0-9\s]/g, '')
      .replace(/\s+/g, '_')
      .substring(0, 50);

    try {
      await nodesAPI.create({
        category,
        node_type: nodeType,
        text,
        semantic_id: semanticId || null,
        display_category: null,
        position_x: null,
        position_y: null,
      });
      await loadGraph();
      setHasChanges(true); // Mark as changed so save button appears
    } catch (err: any) {
      setError(`Failed to create node: ${err.message}`);
      console.error('Error creating node:', err);
    }
  };

  const handleClose = () => {
    if (hasChanges) {
      if (confirm('You have unsaved changes. Close editor? All changes will be lost.')) {
        onClose();
      }
    } else {
      onClose();
    }
  };

  const handleSave = async () => {
    if (!graphData || !hasChanges) {
      onSave();
      return;
    }

    try {
      setLoading(true);
      setError(null);

      // Collect node positions from React Flow
      const nodePositions: Record<string, { x: number; y: number }> = {};
      flowNodes.forEach(node => {
        nodePositions[node.id] = {
          x: node.position.x,
          y: node.position.y,
        };
      });

      // Update node positions in the database
      for (const node of graphData.nodes) {
        const pos = nodePositions[node.id];
        if (pos) {
          await nodesAPI.update(node.id, {
            position_x: pos.x,
            position_y: pos.y,
          });
        }
      }

      // Save layout positions to localStorage as backup
      const layoutKey = `graph_layout_${category}`;
      localStorage.setItem(layoutKey, JSON.stringify(nodePositions));

      setHasChanges(false);
      alert('Graph saved successfully!');
      onSave();
    } catch (err: any) {
      setError(`Failed to save: ${err.message}`);
      console.error('Error saving graph:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white rounded-xl p-8">
          <div className="text-xl font-semibold text-gray-700">Loading graph...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex flex-col">
      {/* Header */}
      <div className="bg-white p-4 shadow-lg">
        <div className="flex justify-between items-start mb-3">
          <div className="flex-1">
            <h2 className="text-2xl font-bold text-gray-800 mb-2">Edit Graph</h2>
            <div className="flex gap-4 items-start">
              <div className="flex-1">
                <label className="block text-xs font-medium text-gray-600 mb-1">Issue Name</label>
                <input
                  type="text"
                  value={editingIssueName}
                  onChange={(e) => {
                    setEditingIssueName(e.target.value);
                    setHasUnsavedIssueChanges(true);
                  }}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  placeholder="e.g., Brush Problems"
                />
              </div>
              <div className="flex-1">
                <label className="block text-xs font-medium text-gray-600 mb-1">Display Category (optional)</label>
                <input
                  type="text"
                  value={editingDisplayCategory}
                  onChange={(e) => {
                    setEditingDisplayCategory(e.target.value);
                    setHasUnsavedIssueChanges(true);
                  }}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm"
                  placeholder="e.g., Electrical, Mechanical, General"
                />
              </div>
            </div>
            <p className="text-xs text-gray-500 mt-2">
              Category ID: <span className="font-mono bg-gray-100 px-2 py-1 rounded">{category}</span> •{' '}
              {graphData?.nodes.length || 0} nodes, {graphData?.connections.length || 0} connections
            </p>
          </div>
          <div className="flex gap-3 ml-4">
            {hasUnsavedIssueChanges && (
              <button
                onClick={handleSaveIssue}
                disabled={loading}
                className="px-4 py-2 rounded-md bg-gradient-to-r from-green-500 to-green-600 text-white font-medium hover:from-green-600 hover:to-green-700 shadow-md text-sm whitespace-nowrap"
              >
                💾 Save Metadata
              </button>
            )}
            <button
              onClick={handleCreateNode}
              className="px-4 py-2 rounded-md bg-green-500 text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium text-sm"
            >
              ➕ New Node
            </button>
            <button
              onClick={handleSave}
              disabled={loading}
              className={`px-6 py-2 rounded-md text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium ${
                hasChanges
                  ? 'bg-gradient-to-br from-[#667eea] to-[#764ba2]'
                  : 'bg-gray-400 cursor-not-allowed'
              }`}
            >
              {loading ? '💾 Saving...' : hasChanges ? '💾 Save Layout *' : '💾 No Changes'}
            </button>
            <button
              onClick={handleClose}
              className="px-6 py-2 rounded-md bg-gray-200 text-gray-700 border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
            >
              ✕ Close
            </button>
          </div>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 m-4 rounded">
          {error}
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 flex relative">
        {/* Node Edit Panel (Left Slide-out) */}
        <div
          className={`absolute left-0 top-0 bottom-0 w-[350px] bg-white border-r border-gray-200 shadow-xl overflow-y-auto z-10 transition-transform duration-300 ease-in-out ${
            openPanel === 'node' ? 'translate-x-0' : '-translate-x-full'
          }`}
        >
          <div className="p-6">
            <h3 className="text-lg font-bold text-gray-800 mb-4">Edit Node</h3>

            {selectedNode ? (
              <div>
                {/* Node Type Selector */}
                <div className="mb-4">
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Node Type
                  </label>
                  <div className="flex gap-3">
                    <button
                      onClick={async () => {
                        if (selectedNode.node_type === 'Question') return;
                        try {
                          await nodesAPI.update(selectedNode.id, { node_type: 'Question' });
                          await loadGraph();
                        } catch (err: any) {
                          setError(`Failed to change node type: ${err.message}`);
                        }
                      }}
                      className={`flex-1 px-4 py-2 rounded-md border-2 font-medium transition-all ${
                        selectedNode.node_type === 'Question'
                          ? 'border-blue-500 bg-blue-100 text-blue-800'
                          : 'border-gray-300 bg-white text-gray-600 hover:border-blue-300'
                      }`}
                    >
                      ❓ Question
                    </button>
                    <button
                      onClick={async () => {
                        if (selectedNode.node_type === 'Conclusion') return;
                        try {
                          await nodesAPI.update(selectedNode.id, { node_type: 'Conclusion' });
                          await loadGraph();
                        } catch (err: any) {
                          setError(`Failed to change node type: ${err.message}`);
                        }
                      }}
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

                {/* Semantic ID */}
                <div className="mb-4">
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Semantic ID
                  </label>
                  <input
                    type="text"
                    value={editingSemanticId}
                    onChange={(e) => {
                      setEditingSemanticId(e.target.value);
                      setHasUnsavedNodeChanges(true);
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md"
                    placeholder="e.g., brush_worn"
                  />
                </div>

                {/* Node Text */}
                <div className="mb-4">
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    {selectedNode.node_type === 'Conclusion' ? 'Conclusion Text' : 'Question Text'}
                  </label>
                  <textarea
                    value={editingText}
                    onChange={(e) => {
                      setEditingText(e.target.value);
                      setHasUnsavedNodeChanges(true);
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md resize-none h-24"
                    placeholder={selectedNode.node_type === 'Conclusion'
                      ? "Enter what to do next..."
                      : "Enter the question..."
                    }
                  />
                </div>

                {/* Save Node Button */}
                {hasUnsavedNodeChanges && (
                  <div className="mb-4">
                    <button
                      onClick={handleSaveNode}
                      className="w-full px-4 py-2 rounded-md bg-gradient-to-r from-green-500 to-green-600 text-white font-medium hover:from-green-600 hover:to-green-700 shadow-md"
                    >
                      Save Node Changes
                    </button>
                  </div>
                )}

                {/* Delete Node Button */}
                <button
                  onClick={() => handleDeleteNode(selectedNode.id)}
                  className="w-full px-3 py-2 rounded-md bg-red-500 text-white font-medium hover:bg-red-600"
                >
                  Delete Node
                </button>
              </div>
            ) : (
              <div className="text-center py-12 text-gray-500">
                <p className="text-sm">No node selected</p>
              </div>
            )}
          </div>
        </div>

        {/* Connection Edit Panel (Right Slide-out) */}
        <div
          className={`absolute right-0 top-0 bottom-0 w-[350px] bg-white border-l border-gray-200 shadow-xl overflow-y-auto z-10 transition-transform duration-300 ease-in-out ${
            openPanel === 'connection' ? 'translate-x-0' : 'translate-x-full'
          }`}
        >
          <div className="p-6">
            <h3 className="text-lg font-bold text-gray-800 mb-4">Edit Connection</h3>

            {selectedConnection ? (
              <div>
                {/* Connection Info */}
                <div className="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-md">
                  <p className="text-xs text-blue-600 font-medium mb-1">From:</p>
                  <p className="text-sm text-gray-800 mb-2">
                    {graphData?.nodes.find(n => n.id === selectedConnection.from_node_id)?.semantic_id ||
                     graphData?.nodes.find(n => n.id === selectedConnection.from_node_id)?.text.substring(0, 30)}
                  </p>
                  <p className="text-xs text-blue-600 font-medium mb-1">To:</p>
                  <p className="text-sm text-gray-800">
                    {graphData?.nodes.find(n => n.id === selectedConnection.to_node_id)?.semantic_id ||
                     graphData?.nodes.find(n => n.id === selectedConnection.to_node_id)?.text.substring(0, 30)}
                  </p>
                </div>

                {/* Connection Label */}
                <div className="mb-4">
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Label
                  </label>
                  <input
                    type="text"
                    value={editingConnections[selectedConnection.id]?.label ?? selectedConnection.label}
                    onChange={(e) => {
                      setEditingConnections(prev => ({
                        ...prev,
                        [selectedConnection.id]: {
                          ...prev[selectedConnection.id],
                          label: e.target.value,
                          to_node_id: prev[selectedConnection.id]?.to_node_id ?? selectedConnection.to_node_id,
                        },
                      }));
                      setHasUnsavedNodeChanges(true);
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md"
                    placeholder="e.g., Yes, No, Worn..."
                  />
                </div>

                {/* Target Node Selector */}
                <div className="mb-4">
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Goes to
                  </label>
                  <select
                    value={editingConnections[selectedConnection.id]?.to_node_id ?? selectedConnection.to_node_id}
                    onChange={(e) => {
                      setEditingConnections(prev => ({
                        ...prev,
                        [selectedConnection.id]: {
                          label: prev[selectedConnection.id]?.label ?? selectedConnection.label,
                          to_node_id: e.target.value,
                        },
                      }));
                      setHasUnsavedNodeChanges(true);
                    }}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md"
                  >
                    {graphData?.nodes
                      .filter(n => n.id !== selectedConnection.from_node_id)
                      .map(n => (
                        <option key={n.id} value={n.id}>
                          {n.node_type === 'Conclusion' ? '🎯 ' : '❓ '}
                          {n.semantic_id || n.text.substring(0, 30)}{n.text.length > 30 ? '...' : ''}
                        </option>
                      ))
                    }
                  </select>
                </div>

                {/* Save Connection Button */}
                {hasUnsavedNodeChanges && (
                  <div className="mb-4">
                    <button
                      onClick={handleSaveConnection}
                      className="w-full px-4 py-2 rounded-md bg-gradient-to-r from-green-500 to-green-600 text-white font-medium hover:from-green-600 hover:to-green-700 shadow-md"
                    >
                      Save Connection Changes
                    </button>
                  </div>
                )}

                {/* Delete Connection Button */}
                <button
                  onClick={() => handleDeleteConnection(selectedConnection.id)}
                  className="w-full px-3 py-2 rounded-md bg-red-500 text-white font-medium hover:bg-red-600"
                >
                  Delete Connection
                </button>
              </div>
            ) : (
              <div className="text-center py-12 text-gray-500">
                <p className="text-sm">No connection selected</p>
              </div>
            )}
          </div>
        </div>

        {/* Graph Visualization (Full Width) */}
        <div className="flex-1 bg-gray-50">
          <ReactFlow
            nodes={flowNodes}
            edges={flowEdges}
            onNodesChange={handleNodesChange}
            onEdgesChange={onFlowEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onEdgeClick={onEdgeClick}
            onPaneClick={onPaneClick}
            nodesDraggable={true}
            fitView
            attributionPosition="bottom-left"
          >
            <Background />
            <Controls />
          </ReactFlow>
        </div>
      </div>
    </div>
  );
}
