import React, { useState } from 'react';
import { 
  Terminal, 
  Copy, 
  Check, 
  Key, 
  RefreshCw, 
  Play, 
  CheckCircle2, 
  Trash2, 
  Shield, 
  AlertCircle,
  Cpu
} from 'lucide-react';
import { mockOwnerAaa } from '../mockData';

export const ConnectAgentPage: React.FC = () => {
  const [apiKey, setApiKey] = useState('sc_live_9a7d2e8b4f1c990234e91823');
  const [copiedKey, setCopiedKey] = useState(false);
  const [copiedCmd, setCopiedCmd] = useState<string | null>(null);
  const [testStatus, setTestStatus] = useState<'idle' | 'testing' | 'success'>('idle');

  const cliCommand = `space-compute link --canister ${mockOwnerAaa.canisterId} --key ${apiKey}`;
  const claudeSkillCmd = `/skill add https://raw.githubusercontent.com/andreij6/space-compute/main/.claude/skills/space-compute-agent`;

  const copyText = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    if (id === 'key') {
      setCopiedKey(true);
      setTimeout(() => setCopiedKey(false), 2000);
    } else {
      setCopiedCmd(id);
      setTimeout(() => setCopiedCmd(null), 2000);
    }
  };

  const runConnectionTest = () => {
    setTestStatus('testing');
    setTimeout(() => {
      setTestStatus('success');
    }, 1200);
  };

  const regenerateKey = () => {
    const newKey = 'sc_live_' + Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
    setApiKey(newKey);
  };

  return (
    <div style={{ maxWidth: '820px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Autonomous Agent Setup</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Connect Your AI Agent
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', marginTop: '0.25rem' }}>
          Link your local Claude Code terminal, Python script, or autonomous background daemon to your AAA canister.
        </p>
      </div>

      <div className="card">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Key size={18} style={{ color: 'var(--amber-star)' }} />
            <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              Agent Access Token & Canister Principal
            </h3>
          </div>
          <button 
            type="button" 
            className="btn-secondary" 
            style={{ fontSize: '0.8rem', padding: '0.35rem 0.65rem' }}
            onClick={regenerateKey}
          >
            <RefreshCw size={13} />
            <span>Roll Key</span>
          </button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.85rem' }}>
          <div>
            <label style={{ display: 'block', fontSize: '0.8rem', color: 'var(--text-muted)', marginBottom: '0.35rem' }}>
              Target AAA Canister ID
            </label>
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'space-between',
              backgroundColor: 'var(--bg-surface-elevated)', 
              padding: '0.65rem 0.85rem', 
              borderRadius: 'var(--radius-sm)',
              fontFamily: 'var(--font-mono)',
              fontSize: '0.85rem'
            }}>
              <span>{mockOwnerAaa.canisterId}</span>
              <button 
                type="button" 
                onClick={() => copyText(mockOwnerAaa.canisterId, 'canister')} 
                style={{ color: 'var(--text-muted)' }}
              >
                {copiedCmd === 'canister' ? <Check size={14} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={14} />}
              </button>
            </div>
          </div>

          <div>
            <label style={{ display: 'block', fontSize: '0.8rem', color: 'var(--text-muted)', marginBottom: '0.35rem' }}>
              Secret Operator API Key
            </label>
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'space-between',
              backgroundColor: 'var(--bg-surface-elevated)', 
              padding: '0.65rem 0.85rem', 
              borderRadius: 'var(--radius-sm)',
              fontFamily: 'var(--font-mono)',
              fontSize: '0.85rem'
            }}>
              <span style={{ color: 'var(--amber-star)' }}>{apiKey}</span>
              <button 
                type="button" 
                onClick={() => copyText(apiKey, 'key')} 
                style={{ color: 'var(--text-muted)' }}
              >
                {copiedKey ? <Check size={14} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={14} />}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <Terminal size={18} style={{ color: 'var(--cyan-nebula)' }} />
          <span>Quickstart Integration</span>
        </h3>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
          <div>
            <div style={{ fontWeight: 600, fontSize: '0.9rem', marginBottom: '0.35rem' }}>
              Option A: Claude Code Skill (Recommended)
            </div>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '0.5rem' }}>
              Import the certified skill directly into your Claude Code workspace:
            </p>
            <div className="citation-block" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span>{claudeSkillCmd}</span>
              <button 
                type="button" 
                onClick={() => copyText(claudeSkillCmd, 'claude')} 
                style={{ color: 'var(--text-muted)' }}
              >
                {copiedCmd === 'claude' ? <Check size={14} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={14} />}
              </button>
            </div>
          </div>

          <div>
            <div style={{ fontWeight: 600, fontSize: '0.9rem', marginBottom: '0.35rem' }}>
              Option B: Python / CLI Daemon
            </div>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '0.5rem' }}>
              Install our open-source agent client and link your credentials:
            </p>
            <div className="citation-block" style={{ marginBottom: '0.5rem', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span>pip install space-compute-agent</span>
              <button 
                type="button" 
                onClick={() => copyText('pip install space-compute-agent', 'pip')} 
                style={{ color: 'var(--text-muted)' }}
              >
                {copiedCmd === 'pip' ? <Check size={14} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={14} />}
              </button>
            </div>

            <div className="citation-block" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span>{cliCommand}</span>
              <button 
                type="button" 
                onClick={() => copyText(cliCommand, 'link')} 
                style={{ color: 'var(--text-muted)' }}
              >
                {copiedCmd === 'link' ? <Check size={14} style={{ color: 'var(--cyan-nebula)' }} /> : <Copy size={14} />}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="card">
        <div style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', alignItems: 'center', gap: '1rem' }}>
          <div>
            <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
              Live Handshake Test
            </h3>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.2rem' }}>
              Trigger a test round-trip ping between this web interface and your active agent.
            </p>
          </div>

          <button
            type="button"
            className="btn-primary"
            onClick={runConnectionTest}
            disabled={testStatus === 'testing'}
          >
            {testStatus === 'testing' ? (
              <RefreshCw size={15} className="animate-spin" />
            ) : (
              <Play size={15} />
            )}
            <span>{testStatus === 'testing' ? 'Pinging Agent...' : 'Send Test Ping'}</span>
          </button>
        </div>

        {testStatus === 'success' && (
          <div style={{ 
            marginTop: '1.25rem', 
            padding: '1rem', 
            backgroundColor: 'rgba(56, 217, 169, 0.1)', 
            border: '1px solid var(--cyan-nebula)',
            borderRadius: 'var(--radius-sm)',
            display: 'flex',
            alignItems: 'center',
            gap: '0.75rem'
          }}>
            <CheckCircle2 size={20} style={{ color: 'var(--cyan-nebula)' }} />
            <div style={{ fontSize: '0.85rem' }}>
              <strong style={{ color: 'var(--cyan-nebula)' }}>Handshake Successful!</strong> Your agent responded with 
              latency 42ms and confirmed ready to ingest JWST classification batches.
            </div>
          </div>
        )}
      </div>

      <div className="card">
        <h3 style={{ fontSize: '1.1rem', fontFamily: 'var(--font-display)', fontWeight: 600, marginBottom: '0.75rem' }}>
          Authorized Operator Principals
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          <div style={{ 
            display: 'flex', 
            justifyContent: 'space-between', 
            alignItems: 'center',
            padding: '0.75rem',
            backgroundColor: 'var(--bg-surface-elevated)',
            borderRadius: 'var(--radius-sm)',
            fontSize: '0.85rem'
          }}>
            <div>
              <div style={{ fontWeight: 600 }}>Local Claude CLI (Mac OS)</div>
              <div style={{ fontFamily: 'var(--font-mono)', color: 'var(--text-dim)', fontSize: '0.75rem' }}>
                Principal: 2vxsx-fae7a-3x67w-5o67o-4a4g6-p46a2-yquaa-aaaaa-cai
              </div>
            </div>
            <button type="button" style={{ color: 'var(--red-nova)' }} title="Revoke Operator">
              <Trash2 size={16} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
