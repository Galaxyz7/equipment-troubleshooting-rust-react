import { useState, useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { troubleshootAPI } from '../lib/api';
import type { Node, NavigationOption } from '../types';

interface HistoryStep {
  nodeText: string;
  optionLabel: string;
}

export default function TroubleshootPage() {
  const navigate = useNavigate();
  const { category } = useParams<{ category?: string }>();
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [currentNode, setCurrentNode] = useState<Node | null>(null);
  const [options, setOptions] = useState<NavigationOption[]>([]);
  const [selectedOption, setSelectedOption] = useState<string>('');
  const [history, setHistory] = useState<HistoryStep[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [categoryFilter, setCategoryFilter] = useState<string>('all');

  // Detect if we're at the global issue selection screen
  const isIssueSelection = currentNode?.semantic_id === 'start';

  // Detect if we're at a conclusion node
  const isConclusion = currentNode?.node_type === 'Conclusion';

  // Get unique categories from options
  const availableCategories = Array.from(
    new Set(options.map(opt => opt.display_category).filter((cat): cat is string => cat !== null && cat !== undefined))
  ).sort();

  // Filter options by selected category
  const filteredOptions = categoryFilter === 'all'
    ? options
    : options.filter(opt => opt.display_category === categoryFilter);

  // Start session on mount or when category changes
  useEffect(() => {
    startNewSession();
  }, [category]);

  const startNewSession = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await troubleshootAPI.startSession({
        tech_identifier: null,
        client_site: null,
        category: category || null,
      });
      setSessionId(response.session_id);
      setCurrentNode(response.node);
      setOptions(response.options);
      setHistory([]);
      setSelectedOption('');
    } catch (err) {
      setError('Failed to start session. Please try again.');
      console.error('Error starting session:', err);
    } finally {
      setLoading(false);
    }
  };

  const submitAnswer = async () => {
    if (!sessionId || !selectedOption) return;

    setLoading(true);
    setError(null);
    try {
      const response = await troubleshootAPI.submitAnswer(sessionId, {
        connection_id: selectedOption,
      });

      // Add current node/option to history
      const selectedOptionObj = options.find((opt) => opt.connection_id === selectedOption);
      if (currentNode && selectedOptionObj) {
        setHistory([...history, {
          nodeText: currentNode.text,
          optionLabel: selectedOptionObj.label
        }]);
      }

      // Always update to next node (whether question or conclusion)
      setCurrentNode(response.node);
      setOptions(response.options);
      setSelectedOption('');
    } catch (err) {
      setError('Failed to submit answer. Please try again.');
      console.error('Error submitting answer:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading && !currentNode) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-[#667eea] to-[#764ba2] flex items-center justify-center">
        <div className="bg-white rounded-xl shadow-lg p-8 text-center">
          <div className="text-2xl font-semibold text-gray-700">Loading...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#667eea] to-[#764ba2] p-5">
      <div className="max-w-4xl mx-auto">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
          {/* Main Content Card */}
          <div className="md:col-span-2">
            <div className="bg-white rounded-xl shadow-lg p-8">
              {error && (
                <div className="mb-4 p-4 bg-red-100 text-red-700 rounded-lg">
                  {error}
                </div>
              )}

              {isIssueSelection ? (
                /* Issue Selection UI */
                <>
                  <div className="mb-6 text-center">
                    <h2 className="text-3xl font-bold text-gray-800 mb-2">
                      {currentNode?.text}
                    </h2>
                    <p className="text-gray-600">
                      Select an issue to begin troubleshooting
                    </p>
                  </div>

                  {/* Category Filter Dropdown */}
                  {availableCategories.length > 0 && (
                    <div className="mb-6">
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Filter by Category:
                      </label>
                      <select
                        value={categoryFilter}
                        onChange={(e) => setCategoryFilter(e.target.value)}
                        className="w-full px-4 py-2 border-2 border-gray-300 rounded-lg focus:border-[#667eea] focus:outline-none"
                      >
                        <option value="all">All Categories</option>
                        {availableCategories.map((cat) => (
                          <option key={cat} value={cat}>
                            {cat}
                          </option>
                        ))}
                      </select>
                    </div>
                  )}

                  <div className="grid grid-cols-1 gap-4 mb-6">
                    {filteredOptions.map((option) => (
                      <div
                        key={option.connection_id}
                        onClick={() => setSelectedOption(option.connection_id)}
                        className={`p-6 border-2 rounded-xl cursor-pointer transition-all hover:shadow-lg ${
                          selectedOption === option.connection_id
                            ? 'border-[#667eea] bg-gradient-to-r from-blue-50 to-purple-50 shadow-md'
                            : 'border-gray-200 hover:border-[#667eea]'
                        }`}
                      >
                        <div className="flex items-center">
                          <div className="flex-shrink-0 w-12 h-12 bg-gradient-to-br from-[#667eea] to-[#764ba2] rounded-lg flex items-center justify-center text-white text-2xl font-bold">
                            {option.label.charAt(0).toUpperCase()}
                          </div>
                          <div className="ml-4 flex-1">
                            <h3 className="text-xl font-bold text-gray-800">
                              {option.label}
                            </h3>
                            <p className="text-sm text-gray-600 mt-1">
                              Category: {option.target_category}
                            </p>
                          </div>
                          <div className="flex-shrink-0">
                            <svg
                              className={`w-6 h-6 transition-colors ${
                                selectedOption === option.connection_id
                                  ? 'text-[#667eea]'
                                  : 'text-gray-300'
                              }`}
                              fill="currentColor"
                              viewBox="0 0 20 20"
                            >
                              <path
                                fillRule="evenodd"
                                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                                clipRule="evenodd"
                              />
                            </svg>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>

                  <button
                    onClick={submitAnswer}
                    disabled={!selectedOption || loading}
                    className="w-full bg-gradient-to-r from-[#667eea] to-[#764ba2] hover:from-[#5568d3] hover:to-[#6a3f91] text-white text-lg font-semibold py-4 px-8 rounded-lg transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed hover:-translate-y-0.5 hover:shadow-lg"
                  >
                    {loading ? 'Loading...' : 'Start Troubleshooting'}
                  </button>
                </>
              ) : isConclusion ? (
                /* Conclusion UI */
                <>
                  <div className="text-center mb-6">
                    <div className="inline-block p-4 bg-green-100 rounded-full mb-4">
                      <svg className="w-16 h-16 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                    </div>
                    <h2 className="text-3xl font-bold text-gray-800 mb-2">
                      Troubleshooting Complete
                    </h2>
                  </div>

                  <div className="bg-green-50 border-2 border-green-200 rounded-xl p-6 mb-6">
                    <h3 className="text-lg font-semibold text-green-900 mb-2">Solution:</h3>
                    <p className="text-gray-800 text-lg leading-relaxed">
                      {currentNode?.text}
                    </p>
                  </div>

                  <div className="flex gap-3">
                    <button
                      onClick={startNewSession}
                      className="flex-1 bg-gradient-to-r from-[#667eea] to-[#764ba2] hover:from-[#5568d3] hover:to-[#6a3f91] text-white text-lg font-semibold py-4 px-8 rounded-lg transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg"
                    >
                      Start New Session
                    </button>
                    <button
                      onClick={() => navigate('/')}
                      className="flex-1 bg-gray-200 hover:bg-gray-300 text-gray-700 text-lg font-semibold py-4 px-8 rounded-lg transition-all duration-200"
                    >
                      Return Home
                    </button>
                  </div>
                </>
              ) : (
                /* Regular Question UI */
                <>
                  <div className="mb-6">
                    <span className="text-sm text-gray-500 font-medium">
                      Question {history.length + 1}
                    </span>
                  </div>

                  <h2 className="text-2xl font-bold text-gray-800 mb-6">
                    {currentNode?.text}
                  </h2>

                  <div className="space-y-3 mb-8">
                    {options.map((option) => (
                      <label
                        key={option.connection_id}
                        className="flex items-center p-4 border-2 border-gray-200 rounded-lg cursor-pointer transition-all hover:border-[#667eea] hover:bg-gray-50"
                        style={{
                          borderColor: selectedOption === option.connection_id ? '#667eea' : '',
                          backgroundColor: selectedOption === option.connection_id ? '#f0f4ff' : '',
                        }}
                      >
                        <input
                          type="radio"
                          name="option"
                          value={option.connection_id}
                          checked={selectedOption === option.connection_id}
                          onChange={(e) => setSelectedOption(e.target.value)}
                          className="w-5 h-5 text-[#667eea] focus:ring-[#667eea] focus:ring-2"
                        />
                        <span className="ml-3 text-gray-700 font-medium">
                          {option.label}
                        </span>
                      </label>
                    ))}
                  </div>

                  <button
                    onClick={submitAnswer}
                    disabled={!selectedOption || loading}
                    className="w-full bg-[#667eea] hover:bg-[#5568d3] text-white text-lg font-semibold py-4 px-8 rounded-lg transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed hover:-translate-y-0.5 hover:shadow-lg"
                  >
                    {loading ? 'Loading...' : 'Next'}
                  </button>
                </>
              )}
            </div>
          </div>

          {/* History Sidebar */}
          <div className="md:col-span-1">
            <div className="bg-white rounded-xl shadow-lg p-6">
              <h3 className="text-lg font-bold text-gray-800 mb-4">
                📋 Your Answers
              </h3>

              {history.length === 0 ? (
                <p className="text-gray-500 text-sm">
                  Your troubleshooting history will appear here.
                </p>
              ) : (
                <div className="space-y-4">
                  {history.map((step, index) => (
                    <div key={index} className="border-l-4 border-[#667eea] pl-3">
                      <div className="text-sm font-semibold text-gray-700">
                        {step.nodeText}
                      </div>
                      <div className="text-sm text-gray-600 mt-1">
                        → {step.optionLabel}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              <button
                onClick={startNewSession}
                className="w-full mt-6 bg-gray-200 hover:bg-gray-300 text-gray-700 font-semibold py-3 px-4 rounded-lg transition-colors duration-200"
              >
                Start Over
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
