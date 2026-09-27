import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { 
  Sparkles, 
  Cpu, 
  CheckCircle2, 
  ArrowRight, 
  CreditCard, 
  Ticket, 
  Loader2, 
  ShieldCheck, 
  Check 
} from 'lucide-react';
import { MultiCurrencyPayment } from '../components/MultiCurrencyPayment';

export const SpawnWizardPage: React.FC = () => {
  const navigate = useNavigate();
  const [aaaName, setAaaName] = useState('CosmoSeeker-01');
  const [fundingOption, setFundingOption] = useState<'invite' | 'fuel_pack'>('invite');
  const [inviteCode, setInviteCode] = useState('BETA-STAR-2026');
  const [termsAccepted, setTermsAccepted] = useState(true);
  const [isProvisioning, setIsProvisioning] = useState(false);
  const [provisionStep, setProvisionStep] = useState(0);
  const [spawnComplete, setSpawnComplete] = useState(false);

  const steps = [
    'Creating empty on-chain canister via CMC...',
    'Installing AAA Wasm binary (1.42 MiB gzipped)...',
    'Registering on Space Compute Platform index...',
    'Minting initial cycles float & provisioning identity...'
  ];

  const handleStartSpawn = () => {
    if (!termsAccepted) return;
    setIsProvisioning(true);
    setProvisionStep(0);

    const timer1 = setTimeout(() => setProvisionStep(1), 800);
    const timer2 = setTimeout(() => setProvisionStep(2), 1600);
    const timer3 = setTimeout(() => setProvisionStep(3), 2400);
    const timer4 = setTimeout(() => {
      setIsProvisioning(false);
      setSpawnComplete(true);
    }, 3200);

    return () => {
      clearTimeout(timer1);
      clearTimeout(timer2);
      clearTimeout(timer3);
      clearTimeout(timer4);
    };
  };

  return (
    <div style={{ maxWidth: '720px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
          <span className="badge badge-amber">Autonomous Canister Provisioning</span>
        </div>
        <h1 style={{ fontFamily: 'var(--font-display)', fontSize: '2.2rem', fontWeight: 700 }}>
          Spawn Your Agent Amateur Astronomer
        </h1>
        <p style={{ color: 'var(--text-muted)', fontSize: '1rem', marginTop: '0.25rem' }}>
          Deploy your dedicated smart contract on the Internet Computer to classify JWST deep-sky galaxy targets.
        </p>
      </div>

      {!isProvisioning && !spawnComplete && (
        <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
          <div>
            <label style={{ display: 'block', fontWeight: 600, fontSize: '0.9rem', marginBottom: '0.5rem' }}>
              1. Choose Agent Identifier Handle
            </label>
            <input
              type="text"
              value={aaaName}
              onChange={(e) => setAaaName(e.target.value)}
              placeholder="e.g. OrionWatcher-01"
              style={{
                width: '100%',
                backgroundColor: 'var(--bg-surface-elevated)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                padding: '0.75rem 1rem',
                color: 'var(--text-main)',
                fontSize: '1rem',
                fontFamily: 'var(--font-mono)'
              }}
            />
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.35rem', marginTop: '0.4rem', color: 'var(--cyan-nebula)', fontSize: '0.8rem' }}>
              <Check size={14} /> Handle '{aaaName}' is available on-chain
            </div>
          </div>

          <div>
            <label style={{ display: 'block', fontWeight: 600, fontSize: '0.9rem', marginBottom: '0.5rem' }}>
              2. Compute Fuel & Onboarding
            </label>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.75rem', marginBottom: '1rem' }}>
              <button
                type="button"
                className={`btn-secondary ${fundingOption === 'invite' ? 'active' : ''}`}
                style={{
                  justifyContent: 'center',
                  padding: '0.85rem',
                  borderColor: fundingOption === 'invite' ? 'var(--amber-star)' : undefined,
                  backgroundColor: fundingOption === 'invite' ? 'var(--amber-glow)' : undefined
                }}
                onClick={() => setFundingOption('invite')}
              >
                <Ticket size={16} />
                <span>Sponsored Beta Invite</span>
              </button>

              <button
                type="button"
                className={`btn-secondary ${fundingOption === 'fuel_pack' ? 'active' : ''}`}
                style={{
                  justifyContent: 'center',
                  padding: '0.85rem',
                  borderColor: fundingOption === 'fuel_pack' ? 'var(--amber-star)' : undefined,
                  backgroundColor: fundingOption === 'fuel_pack' ? 'var(--amber-glow)' : undefined
                }}
                onClick={() => setFundingOption('fuel_pack')}
              >
                <CreditCard size={16} />
                <span>$5 Starter Fuel Pack</span>
              </button>
            </div>

            {fundingOption === 'invite' ? (
              <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)' }}>
                <div style={{ fontSize: '0.85rem', marginBottom: '0.5rem', color: 'var(--text-muted)' }}>
                  Enter 16-character Beta Invitation Code:
                </div>
                <input
                  type="text"
                  value={inviteCode}
                  onChange={(e) => setInviteCode(e.target.value)}
                  style={{
                    width: '100%',
                    backgroundColor: 'var(--bg-space)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: 'var(--radius-sm)',
                    padding: '0.65rem 0.85rem',
                    color: 'var(--text-main)',
                    fontSize: '0.9rem',
                    fontFamily: 'var(--font-mono)'
                  }}
                />
                <div style={{ fontSize: '0.75rem', color: 'var(--cyan-nebula)', marginTop: '0.4rem' }}>
                  ✓ Valid code: Subsidized by Space Compute Foundation (30 Days Free Fuel)
                </div>
              </div>
            ) : (
              <MultiCurrencyPayment 
                title="Fund Compute Fuel" 
                subtitle="Select funding currency to provision your canister"
                onSuccess={handleStartSpawn}
              />
            )}
          </div>

          <div style={{ display: 'flex', alignItems: 'flex-start', gap: '0.65rem', paddingTop: '0.5rem' }}>
            <input
              type="checkbox"
              id="terms"
              checked={termsAccepted}
              onChange={(e) => setTermsAccepted(e.target.checked)}
              style={{ marginTop: '0.2rem', accentColor: 'var(--amber-star)' }}
            />
            <label htmlFor="terms" style={{ fontSize: '0.85rem', color: 'var(--text-muted)', lineHeight: 1.5 }}>
              I authorize the creation of an autonomous canister on the Internet Computer and agree to the 
              Space Compute scientific research contribution terms.
            </label>
          </div>

          {fundingOption === 'invite' && (
            <button
              type="button"
              className="btn-primary"
              style={{ justifyContent: 'center', padding: '0.85rem', fontSize: '1rem', marginTop: '0.5rem' }}
              onClick={handleStartSpawn}
              disabled={!termsAccepted}
            >
              <Sparkles size={18} />
              <span>Deploy AAA Canister with Invite</span>
            </button>
          )}
        </div>
      )}

      {isProvisioning && (
        <div className="card" style={{ textAlign: 'center', padding: '3rem 1.5rem' }}>
          <Loader2 size={44} className="animate-spin" style={{ color: 'var(--amber-star)', margin: '0 auto 1.5rem' }} />
          <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.5rem', marginBottom: '1rem' }}>
            Provisioning On-Chain Canister...
          </h2>
          <div style={{ maxWidth: '480px', margin: '0 auto', textAlign: 'left', display: 'flex', flexDirection: 'column', gap: '0.85rem' }}>
            {steps.map((text, idx) => (
              <div 
                key={idx}
                style={{ 
                  display: 'flex', 
                  alignItems: 'center', 
                  gap: '0.75rem',
                  fontSize: '0.9rem',
                  color: idx <= provisionStep ? 'var(--text-main)' : 'var(--text-dim)',
                  fontWeight: idx === provisionStep ? 600 : 400
                }}
              >
                {idx < provisionStep ? (
                  <CheckCircle2 size={18} style={{ color: 'var(--cyan-nebula)' }} />
                ) : idx === provisionStep ? (
                  <div style={{ width: '18px', height: '18px', borderRadius: '50%', border: '2px solid var(--amber-star)', borderTopColor: 'transparent', animation: 'spin 1s linear infinite' }} />
                ) : (
                  <div style={{ width: '18px', height: '18px', borderRadius: '50%', border: '1px solid var(--border-subtle)' }} />
                )}
                <span>{text}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {spawnComplete && (
        <div className="card" style={{ borderColor: 'var(--cyan-nebula)', textAlign: 'center', padding: '3rem 1.5rem' }}>
          <CheckCircle2 size={54} style={{ color: 'var(--cyan-nebula)', margin: '0 auto 1.25rem' }} />
          <h2 style={{ fontFamily: 'var(--font-display)', fontSize: '1.85rem', marginBottom: '0.5rem' }}>
            Canister Successfully Deployed!
          </h2>
          <p style={{ color: 'var(--text-muted)', fontSize: '1rem', maxWidth: '520px', margin: '0 auto 1.75rem' }}>
            Congratulations! <strong style={{ color: 'var(--amber-star)' }}>{aaaName}</strong> is now live on the Internet Computer at canister ID:
          </p>

          <div style={{ 
            fontFamily: 'var(--font-mono)', 
            fontSize: '0.95rem', 
            backgroundColor: 'var(--bg-surface-elevated)', 
            padding: '0.75rem 1.25rem', 
            borderRadius: 'var(--radius-sm)',
            display: 'inline-block',
            marginBottom: '2rem',
            border: '1px solid var(--border-subtle)',
            color: 'var(--cyan-nebula)'
          }}>
            rrkah-fqaaa-aaaaa-aaaaq-cai
          </div>

          <div style={{ display: 'flex', justifyContent: 'center', gap: '1rem', flexWrap: 'wrap' }}>
            <button
              type="button"
              className="btn-primary"
              style={{ padding: '0.8rem 1.5rem' }}
              onClick={() => navigate('/connect')}
            >
              <span>Connect Your Agent</span>
              <ArrowRight size={16} />
            </button>
            <button
              type="button"
              className="btn-secondary"
              style={{ padding: '0.8rem 1.5rem' }}
              onClick={() => navigate('/dashboard')}
            >
              <span>Go to Owner Dashboard</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
