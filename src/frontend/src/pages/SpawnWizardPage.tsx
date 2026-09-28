import { useEffect, useMemo, useReducer, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useAuth, useMyAaa } from '../auth';
import { paymentsActor, platformActor } from '../ic';
import { Purpose } from '../bindings/payments';
import { PaymentPanel } from '../components/PaymentPanel';
import { unwrapResult } from '../lib/paymentOps';
import {
  avatarGrid,
  checkNameResultMessage,
  browserStore,
  clearSpawnOp,
  initialSpawnState,
  loadSpawnOp,
  randomAvatarSeed,
  resumeSpawnState,
  saveSpawnOp,
  spawnReducer,
  validateNameLocally,
} from '../lib/spawnFlow';
import { CheckNameResult } from '../bindings/platform';

function AvatarPreview({ seed }: { seed: bigint }) {
  const grid = avatarGrid(seed);
  const cell = 16;
  const size = grid.length * cell;
  return (
    <svg viewBox={`0 0 ${size} ${size}`} width={size} height={size} role="img" aria-label={`Avatar preview for seed ${seed}`}>
      <rect x={0} y={0} width={size} height={size} fill="none" />
      {grid.map((row, y) =>
        row.map(
          (on, x) =>
            on && <rect key={`${x}-${y}`} x={x * cell} y={y * cell} width={cell} height={cell} fill="currentColor" />,
        ),
      )}
    </svg>
  );
}

export function SpawnWizardPage() {
  const { identity, principal } = useAuth();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const aaaQuery = useMyAaa();

  const [state, dispatch] = useReducer(spawnReducer, initialSpawnState);
  const [nameInput, setNameInput] = useState('');
  const [debouncedName, setDebouncedName] = useState('');
  const [confirmedName, setConfirmedName] = useState('');
  const [avatarSeed, setAvatarSeed] = useState(randomAvatarSeed);

  const payments = useMemo(() => paymentsActor(identity ?? undefined), [identity]);
  const platform = useMemo(() => platformActor(identity ?? undefined), [identity]);
  const featuresQuery = useQuery({ queryKey: ['payments_features'], queryFn: () => payments.get_features() });

  const principalText = principal?.toText() ?? null;
  const saved = useMemo(() => (principalText ? loadSpawnOp(browserStore(), principalText) : null), [principalText]);
  const resumeOpId = saved?.opId ?? null;
  const resumeOpQuery = useQuery({
    queryKey: ['spawnResumeOp', resumeOpId?.toString()],
    queryFn: () => payments.get_op(resumeOpId as bigint),
    enabled: resumeOpId !== null,
  });

  useEffect(() => {
    if (resumeOpId === null || resumeOpQuery.data === undefined) return;
    const resumed = resumeSpawnState(resumeOpQuery.data);
    if (resumed) dispatch({ type: 'RESUME', state: resumed });
    else if (principalText) clearSpawnOp(browserStore(), principalText);
  }, [resumeOpId, resumeOpQuery.data, principalText]);

  useEffect(() => {
    const t = setTimeout(() => setDebouncedName(nameInput), 400);
    return () => clearTimeout(t);
  }, [nameInput]);

  const localValidation = validateNameLocally(debouncedName);
  const checkNameQuery = useQuery({
    queryKey: ['check_name', localValidation.ok ? localValidation.name : ''],
    queryFn: () => platform.check_name(localValidation.ok ? localValidation.name : ''),
    enabled: localValidation.ok,
  });

  useEffect(() => {
    if (state.step === 'name' && aaaQuery.data) navigate('/dashboard', { replace: true });
  }, [state.step, aaaQuery.data, navigate]);

  useEffect(() => {
    if (state.step !== 'done') return;
    const t = setTimeout(() => navigate('/connect'), 2_500);
    return () => clearTimeout(t);
  }, [state.step, navigate]);

  if (!principal) return <p>Loading…</p>;
  if (aaaQuery.isPending) return <p>Loading…</p>;
  if (aaaQuery.isError) return <p role="alert">Could not check your account. Try again later.</p>;
  if (resumeOpId !== null && resumeOpQuery.isPending) return <p>Loading…</p>;

  const nameTouched = debouncedName.trim().length > 0;
  const nameError = !localValidation.ok
    ? localValidation.message
    : checkNameQuery.data
      ? checkNameResultMessage(checkNameQuery.data)
      : null;
  const nameAvailable = localValidation.ok && checkNameQuery.data === CheckNameResult.Ok;
  const checkingName = localValidation.ok && checkNameQuery.isFetching;

  return (
    <div>
      <h1>Spawn your agent amateur astronomer</h1>
      <p>Your AAA is a canister on the Internet Computer that classifies JWST images on your behalf.</p>

      {state.step === 'name' && (
        <div>
          <p>Step 1 of 3: Name your agent</p>
          <label htmlFor="aaa-name">Agent name</label>
          <input
            id="aaa-name"
            type="text"
            value={nameInput}
            onChange={(e) => setNameInput(e.target.value)}
            autoComplete="off"
          />
          {checkingName && <p>Checking availability…</p>}
          {!checkingName && nameTouched && nameError && <p role="alert">{nameError}</p>}

          <fieldset>
            <legend>Avatar</legend>
            <label htmlFor="avatar-seed">Avatar seed</label>
            <input
              id="avatar-seed"
              type="number"
              min={0}
              value={avatarSeed.toString()}
              onChange={(e) => {
                const n = Number(e.target.value);
                if (Number.isFinite(n) && n >= 0) setAvatarSeed(BigInt(Math.floor(n)));
              }}
            />
            <button type="button" onClick={() => setAvatarSeed(randomAvatarSeed())}>
              Randomize
            </button>
            <AvatarPreview seed={avatarSeed} />
          </fieldset>

          <button
            type="button"
            disabled={!nameAvailable || checkingName}
            onClick={() => {
              setConfirmedName((localValidation.ok && localValidation.name) || '');
              dispatch({ type: 'NAME_CONFIRMED' });
            }}
          >
            Continue
          </button>
        </div>
      )}

      {state.step !== 'name' && (
        <div>
          <p>{state.step === 'done' ? 'Step 3 of 3: Done' : 'Step 2 of 3: Pay to spawn your agent'}</p>
          <p>Agent name: {confirmedName || saved?.name}</p>

          {featuresQuery.isPending && <p>Loading…</p>}
          {featuresQuery.isError && <p role="alert">Could not load payment options. Try again later.</p>}

          {featuresQuery.data && (
            <PaymentPanel
              purpose="spawn"
              beneficiary={principal}
              sponsoredSpawnEnabled={featuresQuery.data.sponsored_spawn}
              initialOpId={state.step === 'progress' || state.step === 'done' ? state.opId : null}
              fetchQuote={() => payments.get_quote_spawn().then(unwrapResult)}
              fetchDepositAccount={() => payments.get_deposit_account(Purpose.Spawn, principal)}
              fetchOp={(opId) => payments.get_op(opId)}
              submitOp={async (path) => {
                const id = await payments
                  .spawn_aaa({ name: confirmedName, avatar_seed: avatarSeed, path })
                  .then(unwrapResult);
                saveSpawnOp(browserStore(), principal.toText(), { opId: id, name: confirmedName });
                dispatch({ type: 'OP_STARTED', opId: id });
                return id;
              }}
              onPaid={async () => {
                dispatch({ type: 'OP_DONE' });
                clearSpawnOp(browserStore(), principal.toText());
                await queryClient.invalidateQueries({ queryKey: ['aaa_by_owner', principal.toText()] });
              }}
            />
          )}

          {state.step === 'done' && (
            <div>
              <p>Your agent amateur astronomer is ready.</p>
              <button type="button" onClick={() => navigate('/connect')}>
                Connect your agent
              </button>
              <button type="button" onClick={() => navigate('/dashboard')}>
                Go to dashboard
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
