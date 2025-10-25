import { useLocation, useNavigate } from 'react-router-dom';
import type { HistoryStep } from '../types/troubleshoot';

export default function ConclusionPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const { conclusion, history } = location.state || {};

  if (!conclusion) {
    navigate('/');
    return null;
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#667eea] to-[#764ba2] p-5">
      <div className="max-w-3xl mx-auto">
        <div className="bg-white rounded-xl shadow-lg p-8 md:p-12">
          {/* Success Icon */}
          <div className="text-center mb-6">
            <div className="inline-flex items-center justify-center w-20 h-20 bg-green-100 rounded-full mb-4">
              <span className="text-5xl">✅</span>
            </div>
            <h1 className="text-3xl font-bold text-gray-800">
              Diagnosis Complete
            </h1>
          </div>

          {/* Conclusion Text */}
          <div className="bg-[#f8f9fa] border-l-4 border-[#667eea] p-5 rounded-lg mb-8">
            <p className="text-gray-800 leading-relaxed whitespace-pre-line text-[1.05em]">
              {conclusion}
            </p>
          </div>

          {/* Action Buttons */}
          <div className="flex flex-col sm:flex-row gap-4 mb-8">
            <button
              onClick={() => navigate('/troubleshoot')}
              className="flex-1 bg-gradient-to-br from-[#667eea] to-[#764ba2] hover:opacity-90 text-white font-semibold py-4 px-6 rounded-lg transition-all duration-200 hover:-translate-y-0.5 hover:shadow-lg"
            >
              Start New Diagnosis
            </button>
            <button
              onClick={() => navigate('/')}
              className="flex-1 bg-gray-200 hover:bg-gray-300 text-gray-700 font-semibold py-4 px-6 rounded-lg transition-colors duration-200"
            >
              Back to Home
            </button>
          </div>
        </div>

        {/* Diagnostic Path */}
        {history && history.length > 0 && (
          <div className="bg-white rounded-xl shadow-lg p-8">
            <h3 className="text-lg font-semibold text-gray-800 mb-4">
              📋 Diagnostic Path
            </h3>
            <div className="space-y-0">
              {history.map((step: HistoryStep, index: number) => (
                <div key={index} className="py-3 border-b border-gray-200 last:border-b-0">
                  <div className="text-gray-600 text-[0.95em] mb-1">
                    {step.question.text}
                  </div>
                  <div className="text-[#667eea] font-semibold">
                    → {step.answer.label}
                  </div>
                </div>
              ))}
            </div>

            {/* Print Button */}
            <button
              onClick={() => window.print()}
              className="mt-4 bg-gray-200 hover:bg-gray-300 text-gray-700 font-normal py-[10px] px-[20px] rounded-lg transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md text-base"
            >
              🖨️ Print Report
            </button>
          </div>
        )}
      </div>

      {/* Print Styles */}
      <style>{`
        @media print {
          body {
            background: white !important;
          }
          button {
            display: none !important;
          }
          a {
            display: none !important;
          }
        }
      `}</style>
    </div>
  );
}
