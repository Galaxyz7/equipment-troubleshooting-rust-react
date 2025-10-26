import { useNavigate } from 'react-router-dom';

export default function LandingPage() {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#667eea] to-[#764ba2] flex items-center justify-center p-5">
      <div className="bg-white rounded-xl shadow-[0_2px_10px_rgba(0,0,0,0.1)] p-10 max-w-[600px] w-full text-center transition-transform duration-200 hover:-translate-y-0.5 hover:shadow-[0_4px_20px_rgba(0,0,0,0.15)]">
        <h1 className="text-4xl font-bold text-gray-800 mb-5">
          🔧 Equipment Troubleshooting
        </h1>
        <p className="text-gray-600 text-lg mb-8 leading-relaxed">
          This tool will guide you through a series of questions to help diagnose and resolve issues with your mechanical equipment. Follow the prompts and answer each question based on your observations.
        </p>
        <button
          onClick={() => navigate('/troubleshoot')}
          className="bg-[#667eea] hover:bg-[#5568d3] text-white text-lg font-semibold py-4 px-8 rounded-lg cursor-pointer transition-all duration-200 hover:-translate-y-0.5 hover:shadow-[0_4px_12px_rgba(102,126,234,0.4)] border-none w-full"
        >
          Start Troubleshooting
        </button>
        <div className="mt-6">
          <a
            href="/login"
            className="text-sm text-[#667eea] hover:underline"
          >
            Admin Login
          </a>
        </div>
      </div>
    </div>
  );
}
