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
  NodeChange,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { issuesAPI, nodesAPI, connectionsAPI } from '../lib/api';
import type { IssueGraph, UpdateNode, UpdateConnection } from '../types';
import { AccessibleAlert } from './AccessibleAlert';
import { AccessibleConfirm } from './AccessibleConfirm';
import { NodeDetailsPanel } from './NodeDetailsPanel';
import { ConnectionDetailsPanel } from './ConnectionDetailsPanel';
import { IssueMetadataHeader } from './IssueMetadataHeader';
import { getErrorMessage } from '../lib/errorUtils';
import { logger } from '../lib/logger';

interface TreeEditorModalProps {
  category: string;
  issueName: string;
  onClose: () => void;
  onSave?: () => void; // Optional - kept for backward compatibility but not used
}

export default function TreeEditorModal({ category, issueName, onClose }: TreeEditorModalProps) {
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
  const [hasUnsavedNodeChanges, setHasUnsavedNodeChanges] = useState(false);

  // Local state for connection editing (track changes by connection ID)
  const [editingConnections, setEditingConnections] = useState<Record<string, { label: string; to_node_id: string }>>({});

  // Local state for issue metadata editing
  const [editingIssueName, setEditingIssueName] = useState<string>(issueName);
  const [editingDisplayCategory, setEditingDisplayCategory] = useState<string>('');
  const [hasUnsavedIssueChanges, setHasUnsavedIssueChanges] = useState(false);
  const [issueData, setIssueData] = useState<any>(null);

  // Accessible dialog states (replaces alert/confirm)
  const [alertDialog, setAlertDialog] = useState<{ isOpen: boolean; title: string; message: string; type?: 'success' | 'info' | 'error' }>({
    isOpen: false,
    title: '',
    message: '',
    type: 'info',
  });
  const [confirmDialog, setConfirmDialog] = useState<{ isOpen: boolean; title: string; message: string; onConfirm: () => void; variant?: 'default' | 'danger' }>({
    isOpen: false,
    title: '',
    message: '',
    onConfirm: () => {},
    variant: 'default',
  });

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
      const isSelected = selectedConnectionId === connection.id;

      reactFlowEdges.push({
        id: connection.id,
        source: connection.from_node_id,
        target: connection.to_node_id,
        label: connection.label,
        type: 'default', // Bezier curves for smooth routing
        animated: false, // Solid lines instead of dashed
        style: {
          stroke: isSelected ? '#8b5cf6' : '#64748b', // Purple when selected, slate gray default
          strokeWidth: isSelected ? 3 : 2, // Thicker when selected
        },
        labelStyle: {
          fill: isSelected ? '#8b5cf6' : '#1f2937',
          fontWeight: isSelected ? 600 : 500,
          fontSize: 12,
        },
        labelBgStyle: {
          fill: '#ffffff',
          fillOpacity: 0.85,
        },
        markerEnd: {
          type: MarkerType.ArrowClosed,
          width: 20,
          height: 20,
          color: isSelected ? '#8b5cf6' : '#64748b',
        },
      });
    });

    setFlowNodes(reactFlowNodes);
    setFlowEdges(reactFlowEdges);
  }, [category, selectedConnectionId, setFlowNodes, setFlowEdges]);

  const loadGraph = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await issuesAPI.getGraph(category);
      setGraphData(data);
      convertGraphToFlow(data);
    } catch (err: unknown) {
      setError(`Failed to load graph: ${getErrorMessage(err)}`);
      logger.error('Failed to load graph data', { category, error: getErrorMessage(err) });
    } finally {
      setLoading(false);
    }
  }, [category, convertGraphToFlow]);

  // Re-render edges when selection changes
  useEffect(() => {
    if (graphData) {
      convertGraphToFlow(graphData);
    }
  }, [selectedConnectionId, graphData, convertGraphToFlow]);

  const loadIssueData = useCallback(async () => {
    try {
      const issues = await issuesAPI.list();
      const issue = issues.find(i => i.category === category);
      if (issue) {
        setIssueData(issue);
      }
    } catch (err: unknown) {
      logger.error('Failed to load issue data', { category, error: getErrorMessage(err) });
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

      const updateData: { name?: string; display_category?: string | null } = {};
      if (editingIssueName !== (issueData?.name || issueName)) {
        updateData.name = editingIssueName;
      }
      if (editingDisplayCategory !== (issueData?.display_category || '')) {
        updateData.display_category = editingDisplayCategory || null;
      }

      if (Object.keys(updateData).length > 0) {
        const updatedIssue = await issuesAPI.update(category, updateData as any);
        setIssueData(updatedIssue);
        setHasUnsavedIssueChanges(false);
        setAlertDialog({
          isOpen: true,
          title: 'Success',
          message: 'Issue metadata saved successfully!',
          type: 'success',
        });
      }
    } catch (err: unknown) {
      setError(`Failed to save issue metadata: ${getErrorMessage(err)}`);
      logger.error('Failed to save issue metadata', { category, error: getErrorMessage(err) });
    } finally {
      setLoading(false);
    }
  };

  // Track node position changes
  const handleNodesChange = useCallback((changes: NodeChange[]) => {
    onFlowNodesChange(changes);
    const hasPositionChange = changes.some((change) =>
      change.type === 'position' && 'dragging' in change && change.dragging === false
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
        // Save current node positions before creating connection
        const nodePositions: Record<string, { x: number; y: number }> = {};
        flowNodes.forEach(node => {
          nodePositions[node.id] = {
            x: node.position.x,
            y: node.position.y,
          };
        });

        // Update positions in database first
        for (const node of graphData.nodes) {
          const pos = nodePositions[node.id];
          if (pos && (node.position_x !== pos.x || node.position_y !== pos.y)) {
            await nodesAPI.update(node.id, {
              position_x: pos.x,
              position_y: pos.y,
            });
          }
        }

        // Count existing connections from source node for order_index
        const existingConnections = graphData.connections.filter(c => c.from_node_id === params.source);

        await connectionsAPI.create({
          from_node_id: params.source,
          to_node_id: params.target,
          label: label.trim(),
          order_index: existingConnections.length,
        });

        // Save positions to localStorage as backup
        const layoutKey = `graph_layout_${category}`;
        localStorage.setItem(layoutKey, JSON.stringify(nodePositions));

        await loadGraph();
        setHasChanges(false);
      } catch (err: unknown) {
        setError(`Failed to create connection: ${getErrorMessage(err)}`);
        logger.error('Failed to create connection', { error: getErrorMessage(err) });
      }
    },
    [graphData, loadGraph, flowNodes, category]
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

      if (Object.keys(nodeUpdates).length > 0) {
        await nodesAPI.update(selectedNode.id, nodeUpdates);
        await loadGraph();
        setHasUnsavedNodeChanges(false);
        setHasChanges(false);
      }
    } catch (err: unknown) {
      setError(`Failed to update node: ${getErrorMessage(err)}`);
      logger.error('Failed to update node', { nodeId: selectedNodeId, error: getErrorMessage(err) });
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
    } catch (err: unknown) {
      setError(`Failed to update connection: ${getErrorMessage(err)}`);
      logger.error('Failed to update connection', { connectionId: selectedConnectionId, error: getErrorMessage(err) });
    }
  };

  // Delete connection
  const handleDeleteConnection = async (connId: string) => {
    setConfirmDialog({
      isOpen: true,
      title: 'Delete Connection',
      message: 'Are you sure you want to delete this connection?',
      variant: 'danger',
      onConfirm: async () => {
        try {
          await connectionsAPI.delete(connId);
          await loadGraph();
          setSelectedConnectionId(null);
          setOpenPanel('none');
          setHasChanges(false);
        } catch (err: unknown) {
          setError(`Failed to delete connection: ${getErrorMessage(err)}`);
          logger.error('Failed to delete connection', { connectionId: selectedConnectionId, error: getErrorMessage(err) });
        }
      },
    });
  };

  // Delete node
  const handleDeleteNode = async (nodeId: string) => {
    const node = graphData?.nodes.find(n => n.id === nodeId);
    if (!node) return;

    setConfirmDialog({
      isOpen: true,
      title: 'Delete Node',
      message: `Delete node "${node.text}"?\n\nThis will also delete all connections.`,
      variant: 'danger',
      onConfirm: async () => {
        try {
          await nodesAPI.delete(nodeId);
          await loadGraph();
          setSelectedNodeId(null);
          setOpenPanel('none');
          setHasChanges(false);
        } catch (err: unknown) {
          setError(`Failed to delete node: ${getErrorMessage(err)}`);
          logger.error('Failed to delete node', { nodeId: selectedNodeId, error: getErrorMessage(err) });
        }
      },
    });
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
      // Save current node positions before creating new node
      if (graphData) {
        const nodePositions: Record<string, { x: number; y: number }> = {};
        flowNodes.forEach(node => {
          nodePositions[node.id] = {
            x: node.position.x,
            y: node.position.y,
          };
        });

        // Update positions in database first
        for (const node of graphData.nodes) {
          const pos = nodePositions[node.id];
          if (pos && (node.position_x !== pos.x || node.position_y !== pos.y)) {
            await nodesAPI.update(node.id, {
              position_x: pos.x,
              position_y: pos.y,
            });
          }
        }

        // Save positions to localStorage as backup
        const layoutKey = `graph_layout_${category}`;
        localStorage.setItem(layoutKey, JSON.stringify(nodePositions));
      }

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
      setHasChanges(false); // Positions already saved
    } catch (err: unknown) {
      setError(`Failed to create node: ${getErrorMessage(err)}`);
      logger.error('Failed to create node', { category, error: getErrorMessage(err) });
    }
  };

  const handleClose = () => {
    if (hasChanges) {
      setConfirmDialog({
        isOpen: true,
        title: 'Unsaved Changes',
        message: 'You have unsaved changes. Close editor?\n\nAll changes will be lost.',
        variant: 'danger',
        onConfirm: () => {
          onClose();
        },
      });
    } else {
      onClose();
    }
  };

  const handleSave = async () => {
    if (!graphData || !hasChanges) {
      // Don't close editor if no changes
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
      setAlertDialog({
        isOpen: true,
        title: 'Success',
        message: 'Graph saved successfully!',
        type: 'success',
      });
      // Don't call onSave() - keep the editor open for continued editing
    } catch (err: unknown) {
      setError(`Failed to save: ${getErrorMessage(err)}`);
      logger.error('Failed to save graph layout', { category, error: getErrorMessage(err) });
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
      <IssueMetadataHeader
        category={category}
        editingIssueName={editingIssueName}
        editingDisplayCategory={editingDisplayCategory}
        hasUnsavedChanges={hasUnsavedIssueChanges}
        nodesCount={graphData?.nodes.length || 0}
        connectionsCount={graphData?.connections.length || 0}
        loading={loading}
        onIssueNameChange={(name) => {
          setEditingIssueName(name);
          setHasUnsavedIssueChanges(true);
        }}
        onDisplayCategoryChange={(cat) => {
          setEditingDisplayCategory(cat);
          setHasUnsavedIssueChanges(true);
        }}
        onSaveMetadata={handleSaveIssue}
        onCreateNode={handleCreateNode}
        onSaveLayout={handleSave}
        onClose={handleClose}
        hasLayoutChanges={hasChanges}
      />

      {/* Error Display */}
      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 m-4 rounded">
          {error}
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 flex relative">
        {/* Node Edit Panel (Left Slide-out) */}
        <NodeDetailsPanel
          isOpen={openPanel === 'node'}
          selectedNode={selectedNode}
          editingText={editingText}
          hasUnsavedChanges={hasUnsavedNodeChanges}
          onEditingTextChange={(text) => {
            setEditingText(text);
            setHasUnsavedNodeChanges(true);
          }}
          onSave={handleSaveNode}
          onDelete={handleDeleteNode}
          onClose={() => setOpenPanel('none')}
          onNodeTypeChange={loadGraph}
          setError={setError}
        />

        {/* Connection Edit Panel (Right Slide-out) */}
        <ConnectionDetailsPanel
          isOpen={openPanel === 'connection'}
          selectedConnection={selectedConnection}
          graphData={graphData}
          editingConnections={editingConnections}
          hasUnsavedChanges={hasUnsavedNodeChanges}
          onConnectionChange={(connId, label, toNodeId) => {
            setEditingConnections(prev => ({
              ...prev,
              [connId]: { label, to_node_id: toNodeId },
            }));
            setHasUnsavedNodeChanges(true);
          }}
          onSave={handleSaveConnection}
          onDelete={handleDeleteConnection}
          onClose={() => setOpenPanel('none')}
        />

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

      {/* Accessible dialogs (replaces alert/confirm) */}
      <AccessibleAlert
        isOpen={alertDialog.isOpen}
        onClose={() => setAlertDialog({ ...alertDialog, isOpen: false })}
        title={alertDialog.title}
        message={alertDialog.message}
        type={alertDialog.type}
      />

      <AccessibleConfirm
        isOpen={confirmDialog.isOpen}
        onClose={() => setConfirmDialog({ ...confirmDialog, isOpen: false })}
        onConfirm={confirmDialog.onConfirm}
        title={confirmDialog.title}
        message={confirmDialog.message}
        variant={confirmDialog.variant}
      />
    </div>
  );
}
