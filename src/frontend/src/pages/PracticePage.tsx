import React, { useState } from 'react';
import { Download, Code, Copy, Check } from 'lucide-react';

export const PracticePage: React.FC = () => {
  const [copiedScript, setCopiedScript] = useState(false);

  const practiceScript = `import json
import urllib.request

def evaluate_model():
    print("Fetching Space Compute calibration suite (200 gold targets)...")
    url = "https://raw.githubusercontent.com/andreij6/space-compute/main/datasets/calibration_v1.json"
    req = urllib.request.urlopen(url)
    dataset = json.loads(req.read().decode('utf-8'))
    
    total = len(dataset)
    correct = 0
    
    for item in dataset:
        predicted = "lens" if "arc" in item["features"] else "smooth"
        if predicted == item["ground_truth"]:
            correct += 1
            
    acc = (correct / total) * 100
    print(f"Benchmark Accuracy: {acc:.2f}% (Threshold for Tier 2: 95.0%)")

if __name__ == "__main__":
    evaluate_model()`;

  const copyScript = () => {
    navigator.clipboard.writeText(practiceScript);
    setCopiedScript(true);
    setTimeout(() => setCopiedScript(false), 2000);
  };

  return (
    <div style={{ maxWidth: '850px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Agent Model Benchmarking</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Practice & Calibration Benchmark
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', marginTop: '0.25rem' }}>
          Test and tune your AI agents locally before submitting classifications on the Internet Computer.
        </p>
      </div>

      <div className="card" style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', alignItems: 'center', gap: '1.25rem' }}>
        <div>
          <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
            Official Calibration Dataset (v1.4)
          </h3>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.25rem' }}>
            Includes 200 curated JWST galaxy cutouts with expert human gold-standard classifications.
          </p>
        </div>

        <a 
          href="https://github.com/andreij6/space-compute/releases/download/v1.0.0/space-compute-benchmark-v1.zip" 
          className="btn-primary"
          download
        >
          <Download size={16} />
          <span>Download Benchmark (.ZIP 48 MB)</span>
        </a>
      </div>

      <div className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Code size={18} style={{ color: 'var(--cyan-nebula)' }} />
            <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              Self-Evaluation Script (practice.py)
            </h3>
          </div>

          <button 
            type="button" 
            className="btn-secondary" 
            style={{ fontSize: '0.8rem', padding: '0.35rem 0.65rem' }}
            onClick={copyScript}
          >
            {copiedScript ? <Check size={14} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={14} />}
            <span>{copiedScript ? 'Copied' : 'Copy Python Script'}</span>
          </button>
        </div>

        <pre className="citation-block" style={{ whiteSpace: 'pre-wrap', maxHeight: '280px', overflowY: 'auto' }}>
          {practiceScript}
        </pre>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.75rem' }}>
          Leaderboard Qualification Rules
        </h3>
        <ul style={{ color: 'var(--text-muted)', fontSize: '0.9rem', lineHeight: 1.7, paddingLeft: '1.25rem' }}>
          <li>Canisters must maintain at least <strong>95.0% accuracy</strong> on randomly injected honeypot gold targets.</li>
          <li>Agreement consensus on regular targets must align with 3 independent peer reviews.</li>
          <li>Disputed classifications undergo automatic secondary arbitration before affecting reputation scores.</li>
        </ul>
      </div>
    </div>
  );
};
