import { useState, useRef, memo } from 'react';
import { issuesAPI } from '../lib/api';
import type { IssueExportData, ImportResult } from '../types/issues';
import { getErrorMessage } from '../lib/errorUtils';
import { logger } from '../lib/logger';

interface ImportModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

const ImportModal = memo(function ImportModal({ isOpen, onClose, onSuccess }: ImportModalProps) {
  const [file, setFile] = useState<File | null>(null);
  const [importing, setImporting] = useState(false);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (selectedFile) {
      if (selectedFile.type !== 'application/json' && !selectedFile.name.endsWith('.json')) {
        setError('Please select a JSON file');
        setFile(null);
        return;
      }
      setFile(selectedFile);
      setError(null);
      setResult(null);
    }
  };

  const handleImport = async () => {
    if (!file) {
      setError('Please select a file first');
      return;
    }

    setImporting(true);
    setError(null);
    setResult(null);

    try {
      const text = await file.text();
      const data = JSON.parse(text) as IssueExportData[];

      // Validate structure
      if (!Array.isArray(data)) {
        throw new Error('Invalid file format: Expected an array of issues');
      }

      // Import issues
      const importResult = await issuesAPI.importIssues(data);
      setResult(importResult);

      // If all successful, close after a delay
      if (importResult.success.length > 0 && importResult.errors.length === 0) {
        setTimeout(() => {
          onSuccess();
        }, 2000);
      }
    } catch (err: unknown) {
      if (err instanceof SyntaxError) {
        setError('Invalid JSON file format');
      } else {
        setError(getErrorMessage(err) || 'Failed to import issues. Please try again.');
      }
      logger.error('Failed to import issues from file', { fileName: file?.name, error: getErrorMessage(err) });
    } finally {
      setImporting(false);
    }
  };

  const handleClose = () => {
    setFile(null);
    setResult(null);
    setError(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-5">
      <div
        className="bg-white rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-y-auto"
        role="dialog"
        aria-modal="true"
        aria-labelledby="import-modal-title"
      >
        <div className="p-8">
          <div className="flex justify-between items-center mb-6">
            <h2 id="import-modal-title" className="text-2xl font-bold text-gray-800"><span aria-hidden="true">📥</span> Import Issues</h2>
            <button
              onClick={handleClose}
              className="text-gray-400 hover:text-gray-600 text-2xl leading-none"
              aria-label="Close import dialog"
            >
              ×
            </button>
          </div>

          {/* Instructions */}
          <div className="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <h3 className="font-semibold text-gray-800 mb-2">Instructions:</h3>
            <ul className="text-sm text-gray-700 space-y-1 list-disc list-inside">
              <li>Select a JSON file exported from this system</li>
              <li>The file should contain one or more issues with their decision trees</li>
              <li>Issues with existing categories will be skipped (delete them first if you want to replace)</li>
              <li>All nodes and connections will be imported atomically (all or nothing per issue)</li>
            </ul>
          </div>

          {/* File Upload */}
          <div className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Select JSON File
            </label>
            <input
              ref={fileInputRef}
              type="file"
              accept=".json,application/json"
              onChange={handleFileChange}
              className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-[#667eea] focus:border-transparent"
            />
            {file && (
              <p className="mt-2 text-sm text-gray-600">
                Selected: {file.name} ({(file.size / 1024).toFixed(2)} KB)
              </p>
            )}
          </div>

          {/* Error Display */}
          {error && (
            <div className="mb-6 p-4 bg-red-50 text-red-700 rounded-lg border border-red-200">
              {error}
            </div>
          )}

          {/* Import Result */}
          {result && (
            <div className="mb-6 space-y-4">
              {/* Success Section */}
              {result.success.length > 0 && (
                <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                  <h3 className="font-semibold text-green-800 mb-2">
                    ✅ Successfully Imported ({result.success.length})
                  </h3>
                  <div className="space-y-2">
                    {result.success.map((success, idx: number) => (
                      <div key={idx} className="text-sm text-green-700">
                        <strong>{success.name}</strong> ({success.category}): {success.nodes_count} nodes, {success.connections_count} connections
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Error Section */}
              {result.errors.length > 0 && (
                <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
                  <h3 className="font-semibold text-red-800 mb-2">
                    ❌ Failed to Import ({result.errors.length})
                  </h3>
                  <div className="space-y-2">
                    {result.errors.map((error, idx: number) => (
                      <div key={idx} className="text-sm text-red-700">
                        <strong>{error.category}</strong>: {error.error}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-3 justify-end">
            <button
              onClick={handleClose}
              className="px-6 py-3 rounded-md bg-gray-200 text-gray-700 border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium"
            >
              {result?.success.length && result.errors.length === 0 ? 'Done' : 'Cancel'}
            </button>
            <button
              onClick={handleImport}
              disabled={!file || importing}
              className="px-6 py-3 rounded-md bg-gradient-to-br from-[#667eea] to-[#764ba2] text-white border-none cursor-pointer transition-transform duration-200 hover:-translate-y-0.5 font-medium disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {importing ? 'Importing...' : 'Import'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
});

export default ImportModal;
