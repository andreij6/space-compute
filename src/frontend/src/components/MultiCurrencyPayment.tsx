import React, { useState } from 'react';
import { CreditCard, QrCode, ArrowRight, CheckCircle2, Loader2 } from 'lucide-react';

interface MultiCurrencyPaymentProps {
  onSuccess?: () => void;
  title?: string;
  subtitle?: string;
}

export const MultiCurrencyPayment: React.FC<MultiCurrencyPaymentProps> = ({
  onSuccess,
  title = 'Select Compute Fuel Pack',
  subtitle = 'Provide cycles fuel to power your AAA canister on the Internet Computer'
}) => {
  const [selectedMethod, setSelectedMethod] = useState<'icp' | 'card' | 'btc' | 'eth'>('icp');
  const [selectedPack, setSelectedPack] = useState<'basic' | 'pro' | 'annual'>('basic');
  const [step, setStep] = useState<'select' | 'processing' | 'confirmed'>('select');

  const packs = [
    {
      id: 'basic',
      name: 'Starter Orbit',
      usdPrice: 5,
      icpPrice: 0.45,
      durationDays: 30,
      cycles: '20 TCycles',
      desc: 'Ideal for 1 AAA classifying up to 500 images/day'
    },
    {
      id: 'pro',
      name: 'Deep Survey',
      usdPrice: 15,
      icpPrice: 1.35,
      durationDays: 90,
      cycles: '65 TCycles',
      badge: 'Popular',
      desc: 'Continuous autonomous peer-review and priority discovery flagging'
    },
    {
      id: 'annual',
      name: 'Observatory Season',
      usdPrice: 50,
      icpPrice: 4.5,
      durationDays: 365,
      cycles: '250 TCycles',
      badge: 'Best Value',
      desc: 'Full year of uninterrupted agent autonomy and high-tier verification'
    }
  ];

  const currentPack = packs.find(p => p.id === selectedPack) || packs[0];

  const handlePay = () => {
    setStep('processing');
    setTimeout(() => {
      setStep('confirmed');
      if (onSuccess) {
        setTimeout(onSuccess, 1500);
      }
    }, 1800);
  };

  return (
    <div className="card" style={{ borderColor: 'var(--border-active)' }}>
      <div style={{ marginBottom: '1.25rem' }}>
        <h3 style={{ fontSize: '1.15rem', fontFamily: 'var(--font-display)', fontWeight: 600 }}>
          {title}
        </h3>
        <p style={{ color: 'var(--text-muted)', fontSize: '0.875rem', marginTop: '0.25rem' }}>
          {subtitle}
        </p>
      </div>

      {step === 'select' && (
        <div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '0.85rem', marginBottom: '1.25rem' }}>
            {packs.map((pack) => {
              const isSelected = selectedPack === pack.id;
              return (
                <div
                  key={pack.id}
                  onClick={() => setSelectedPack(pack.id as typeof selectedPack)}
                  style={{
                    padding: '1rem',
                    borderRadius: 'var(--radius-sm)',
                    border: `1.5px solid ${isSelected ? 'var(--amber-star)' : 'var(--border-subtle)'}`,
                    backgroundColor: isSelected ? 'var(--amber-glow)' : 'var(--bg-surface-elevated)',
                    cursor: 'pointer',
                    transition: 'all 0.15s ease',
                    position: 'relative'
                  }}
                >
                  {pack.badge && (
                    <span 
                      className="badge badge-amber" 
                      style={{ position: 'absolute', top: '0.5rem', right: '0.5rem', fontSize: '0.65rem' }}
                    >
                      {pack.badge}
                    </span>
                  )}
                  <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>{pack.name}</div>
                  <div style={{ fontSize: '1.25rem', fontFamily: 'var(--font-display)', fontWeight: 700, margin: '0.35rem 0', color: 'var(--text-main)' }}>
                    ${pack.usdPrice} <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)', fontWeight: 400 }}>/ {pack.durationDays}d</span>
                  </div>
                  <div style={{ fontSize: '0.8rem', color: 'var(--amber-star)', fontWeight: 500 }}>
                    {pack.cycles}
                  </div>
                  <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginTop: '0.5rem', lineHeight: 1.4 }}>
                    {pack.desc}
                  </div>
                </div>
              );
            })}
          </div>

          <div style={{ marginBottom: '1.25rem' }}>
            <label style={{ display: 'block', fontSize: '0.85rem', fontWeight: 600, color: 'var(--text-muted)', marginBottom: '0.5rem' }}>
              Payment Method
            </label>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(130px, 1fr))', gap: '0.5rem' }}>
              <button
                type="button"
                className={`btn-secondary ${selectedMethod === 'icp' ? 'active' : ''}`}
                style={{
                  justifyContent: 'center',
                  borderColor: selectedMethod === 'icp' ? 'var(--amber-star)' : undefined,
                  backgroundColor: selectedMethod === 'icp' ? 'var(--amber-glow)' : undefined
                }}
                onClick={() => setSelectedMethod('icp')}
              >
                <span>ICP Wallet</span>
              </button>

              <button
                type="button"
                className={`btn-secondary ${selectedMethod === 'card' ? 'active' : ''}`}
                style={{
                  justifyContent: 'center',
                  borderColor: selectedMethod === 'card' ? 'var(--amber-star)' : undefined,
                  backgroundColor: selectedMethod === 'card' ? 'var(--amber-glow)' : undefined
                }}
                onClick={() => setSelectedMethod('card')}
              >
                <CreditCard size={15} />
                <span>Card (Stripe)</span>
              </button>

              <button
                type="button"
                className={`btn-secondary ${selectedMethod === 'btc' ? 'active' : ''}`}
                style={{
                  justifyContent: 'center',
                  borderColor: selectedMethod === 'btc' ? 'var(--amber-star)' : undefined,
                  backgroundColor: selectedMethod === 'btc' ? 'var(--amber-glow)' : undefined
                }}
                onClick={() => setSelectedMethod('btc')}
              >
                <QrCode size={15} />
                <span>ckBTC</span>
              </button>

              <button
                type="button"
                className={`btn-secondary ${selectedMethod === 'eth' ? 'active' : ''}`}
                style={{
                  justifyContent: 'center',
                  borderColor: selectedMethod === 'eth' ? 'var(--amber-star)' : undefined,
                  backgroundColor: selectedMethod === 'eth' ? 'var(--amber-glow)' : undefined
                }}
                onClick={() => setSelectedMethod('eth')}
              >
                <span>ckETH</span>
              </button>
            </div>
          </div>

          <div style={{ backgroundColor: 'var(--bg-surface-elevated)', padding: '1rem', borderRadius: 'var(--radius-sm)', marginBottom: '1.25rem' }}>
            {selectedMethod === 'icp' && (
              <div style={{ fontSize: '0.85rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.35rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Direct Canister Funding:</span>
                  <span style={{ fontWeight: 600 }}>{currentPack.icpPrice} ICP</span>
                </div>
                <div style={{ color: 'var(--text-dim)', fontSize: '0.75rem' }}>
                  Converted automatically to {currentPack.cycles} via Cycles Minting Canister (CMC).
                </div>
              </div>
            )}

            {selectedMethod === 'card' && (
              <div style={{ fontSize: '0.85rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.35rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Stripe Fiat Checkout:</span>
                  <span style={{ fontWeight: 600 }}>${currentPack.usdPrice}.00 USD</span>
                </div>
                <div style={{ color: 'var(--text-dim)', fontSize: '0.75rem' }}>
                  Payments canister converts fiat through treasury float into canister cycles in seconds.
                </div>
              </div>
            )}

            {selectedMethod === 'btc' && (
              <div style={{ fontSize: '0.85rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.35rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Bitcoin Settlement:</span>
                  <span style={{ fontWeight: 600 }}>0.000078 BTC</span>
                </div>
                <div style={{ color: 'var(--text-dim)', fontSize: '0.75rem' }}>
                  Deposit to on-chain canister Bitcoin address, credited upon 1 confirmation.
                </div>
              </div>
            )}

            {selectedMethod === 'eth' && (
              <div style={{ fontSize: '0.85rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.35rem' }}>
                  <span style={{ color: 'var(--text-muted)' }}>Ethereum Settlement:</span>
                  <span style={{ fontWeight: 600 }}>0.0018 ETH</span>
                </div>
                <div style={{ color: 'var(--text-dim)', fontSize: '0.75rem' }}>
                  Native EVM RPC integration on Internet Computer.
                </div>
              </div>
            )}
          </div>

          <button
            type="button"
            className="btn-primary"
            style={{ width: '100%', justifyContent: 'center', padding: '0.75rem' }}
            onClick={handlePay}
          >
            <span>Confirm & Refuel ({selectedMethod.toUpperCase()})</span>
            <ArrowRight size={16} />
          </button>
        </div>
      )}

      {step === 'processing' && (
        <div style={{ textAlign: 'center', padding: '2rem 1rem' }}>
          <Loader2 size={36} className="animate-spin" style={{ color: 'var(--amber-star)', margin: '0 auto 1rem' }} />
          <h4 style={{ fontSize: '1.1rem', marginBottom: '0.5rem' }}>Minting Cycles on Internet Computer...</h4>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', maxWidth: '360px', margin: '0 auto' }}>
            Routing payment through CMC, topping up canister cycles balance and generating certified proof.
          </p>
        </div>
      )}

      {step === 'confirmed' && (
        <div style={{ textAlign: 'center', padding: '2rem 1rem' }}>
          <CheckCircle2 size={40} style={{ color: 'var(--cyan-nebula)', margin: '0 auto 1rem' }} />
          <h4 style={{ fontSize: '1.1rem', marginBottom: '0.5rem' }}>Canister Fuel Deposited!</h4>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', maxWidth: '360px', margin: '0 auto 1.25rem' }}>
            Successfully injected {currentPack.cycles}. Your AAA canister now has {currentPack.durationDays} days of active autonomy.
          </p>
          <button
            type="button"
            className="btn-secondary"
            onClick={() => setStep('select')}
          >
            Done
          </button>
        </div>
      )}
    </div>
  );
};
