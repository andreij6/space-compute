import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useAuth } from '../auth';
import { paymentsActor } from '../ic';
import { Purpose } from '../bindings/payments';
import { PaymentPanel } from '../components/PaymentPanel';
import { unwrapResult } from '../lib/paymentOps';

function randomAvatarSeed(): bigint {
  return BigInt(Math.floor(Math.random() * 1_000_000_000));
}

export function SpawnWizardPage() {
  const { identity, principal } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [name, setName] = useState('');
  const [avatarSeed] = useState(randomAvatarSeed);

  const payments = useMemo(() => paymentsActor(identity ?? undefined), [identity]);
  const featuresQuery = useQuery({ queryKey: ['payments_features'], queryFn: () => payments.get_features() });

  if (!principal) return <p>Loading…</p>;
  const trimmedName = name.trim();

  return (
    <div>
      <h1>Spawn your agent amateur astronomer</h1>
      <p>Your AAA is a canister on the Internet Computer that classifies JWST images on your behalf.</p>

      <label htmlFor="aaa-name">Agent name</label>
      <input id="aaa-name" type="text" value={name} onChange={(e) => setName(e.target.value)} autoComplete="off" />

      {featuresQuery.isPending && <p>Loading…</p>}
      {featuresQuery.isError && <p role="alert">Could not load payment options. Try again later.</p>}

      {trimmedName.length > 0 && featuresQuery.data && (
        <PaymentPanel
          purpose="spawn"
          beneficiary={principal}
          sponsoredSpawnEnabled={featuresQuery.data.sponsored_spawn}
          fetchQuote={() => payments.get_quote_spawn().then(unwrapResult)}
          fetchDepositAccount={() => payments.get_deposit_account(Purpose.Spawn, principal)}
          fetchOp={(opId) => payments.get_op(opId)}
          submitOp={(path) => payments.spawn_aaa({ name: trimmedName, avatar_seed: avatarSeed, path }).then(unwrapResult)}
          onPaid={async () => {
            await Promise.all([
              queryClient.invalidateQueries({ queryKey: ['aaa_by_owner', principal.toText()] }),
              new Promise((resolve) => setTimeout(resolve, 2_000)),
            ]);
            navigate('/dashboard');
          }}
        />
      )}
    </div>
  );
}
