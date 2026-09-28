import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Principal } from '@icp-sdk/core/principal';
import { AdminPageShell } from '../components/AdminPageShell';
import { ConfirmAction } from '../components/ConfirmAction';
import { ParamsEditor } from '../components/ParamsEditor';
import { Button } from '../components/ui/Button';
import { useAuth } from '../auth';
import { platformActor, paymentsActor, treasuryActor } from '../ic';
import { canRemoveAdmin, unwrapAdmin } from '../lib/admin';
import type { Params as PlatformParams } from '../bindings/platform';
import type { Features, Params as PaymentsParams } from '../bindings/payments';
import card from '../components/ui/Card.module.css';
import shared from '../styles/adminShared.module.css';

function AdminList({
  title,
  admins,
  onAdd,
  onRemove,
  adding,
  removing,
}: {
  title: string;
  admins: Principal[];
  onAdd: (p: Principal) => void;
  onRemove: (p: Principal) => void;
  adding: boolean;
  removing: boolean;
}) {
  const [newAdmin, setNewAdmin] = useState('');
  return (
    <div className={card.card}>
      <h3>{title}</h3>
      <ul className={shared.detailList}>
        {admins.map((p) => (
          <li key={p.toText()} className={shared.row}>
            {p.toText()}{' '}
            <ConfirmAction
              label="remove"
              phrase={p.toText().slice(0, 5)}
              disabled={removing || !canRemoveAdmin(admins)}
              onConfirm={() => onRemove(p)}
            />
          </li>
        ))}
      </ul>
      <div className={shared.formRow}>
        <label className={shared.field}>
          New admin principal
          <input className={shared.input} value={newAdmin} onChange={(e) => setNewAdmin(e.target.value)} />
        </label>
        <Button
          variant="secondary"
          disabled={adding || !newAdmin.trim()}
          onClick={() => {
            onAdd(Principal.fromText(newAdmin.trim()));
            setNewAdmin('');
          }}
        >
          Add admin
        </Button>
      </div>
    </div>
  );
}

export const AdminSettingsPage: React.FC = () => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();

  const platformOverview = useQuery({
    queryKey: ['admin', 'platform', 'overview'],
    queryFn: async () => unwrapAdmin(await platformActor(identity!).admin_overview()),
  });
  const paymentsOverview = useQuery({
    queryKey: ['admin', 'payments', 'overview'],
    queryFn: async () => unwrapAdmin(await paymentsActor(identity!).admin_overview()),
  });
  const treasuryConfig = useQuery({
    queryKey: ['admin', 'treasury', 'config'],
    queryFn: async () => unwrapAdmin(await treasuryActor(identity!).admin_get_config()),
  });

  const invalidatePlatform = () => queryClient.invalidateQueries({ queryKey: ['admin', 'platform', 'overview'] });
  const invalidatePayments = () => queryClient.invalidateQueries({ queryKey: ['admin', 'payments', 'overview'] });
  const invalidateTreasury = () => queryClient.invalidateQueries({ queryKey: ['admin', 'treasury', 'config'] });

  const setPlatformParams = useMutation({
    mutationFn: async (params: PlatformParams) => unwrapAdmin(await platformActor(identity!).admin_set_params(params)),
    onSuccess: invalidatePlatform,
  });
  const setPaymentsParams = useMutation({
    mutationFn: async (params: PaymentsParams) => unwrapAdmin(await paymentsActor(identity!).admin_set_params(params)),
    onSuccess: invalidatePayments,
  });
  const setFeatures = useMutation({
    mutationFn: async (features: Features) => unwrapAdmin(await paymentsActor(identity!).admin_set_features(features)),
    onSuccess: invalidatePayments,
  });

  const addPlatformAdmin = useMutation({
    mutationFn: async (p: Principal) => unwrapAdmin(await platformActor(identity!).admin_add_admin(p)),
    onSuccess: invalidatePlatform,
  });
  const removePlatformAdmin = useMutation({
    mutationFn: async (p: Principal) => unwrapAdmin(await platformActor(identity!).admin_remove_admin(p)),
    onSuccess: invalidatePlatform,
  });
  const addPaymentsAdmin = useMutation({
    mutationFn: async (p: Principal) => unwrapAdmin(await paymentsActor(identity!).admin_add_admin(p)),
    onSuccess: invalidatePayments,
  });
  const removePaymentsAdmin = useMutation({
    mutationFn: async (p: Principal) => unwrapAdmin(await paymentsActor(identity!).admin_remove_admin(p)),
    onSuccess: invalidatePayments,
  });
  const setTreasuryAdmins = useMutation({
    mutationFn: async (admins: Principal[]) =>
      unwrapAdmin(await treasuryActor(identity!).admin_set_config({ ...treasuryConfig.data!, admins })),
    onSuccess: invalidateTreasury,
  });

  return (
    <AdminPageShell title="Settings, feature flags & admins">
      <section aria-label="Platform params" className={card.card}>
        <h2 className={card.title}>Platform params</h2>
        {platformOverview.isPending && <p>Loading…</p>}
        {platformOverview.isError && <p role="alert" className={shared.alert}>{platformOverview.error.message}</p>}
        {platformOverview.data && (
          <ParamsEditor
            base={platformOverview.data.params}
            floatKeys={['claim_cell_arcsec']}
            onSave={(p) => setPlatformParams.mutate(p)}
            saving={setPlatformParams.isPending}
            error={setPlatformParams.error?.message}
          />
        )}
      </section>

      <section aria-label="Payments params" className={card.card}>
        <h2 className={card.title}>Payments params</h2>
        {paymentsOverview.isPending && <p>Loading…</p>}
        {paymentsOverview.isError && <p role="alert" className={shared.alert}>{paymentsOverview.error.message}</p>}
        {paymentsOverview.data && (
          <ParamsEditor
            base={paymentsOverview.data.params}
            onSave={(p) => setPaymentsParams.mutate(p)}
            saving={setPaymentsParams.isPending}
            error={setPaymentsParams.error?.message}
          />
        )}
      </section>

      <section aria-label="Feature flags (ICP-only MVP: card/BTC/ETH stay pause-only)" className={card.card}>
        <h2 className={card.title}>Feature flags</h2>
        {paymentsOverview.data && (
          <>
            {(['card', 'btc', 'eth', 'sponsored_spawn'] as const).map((flag) => (
              <p key={flag} className={shared.row}>
                {flag}: {paymentsOverview.data.features[flag] ? 'enabled' : 'disabled'}{' '}
                <ConfirmAction
                  label={paymentsOverview.data.features[flag] ? `disable ${flag}` : `enable ${flag}`}
                  phrase={flag}
                  disabled={setFeatures.isPending}
                  onConfirm={() =>
                    setFeatures.mutate({ ...paymentsOverview.data!.features, [flag]: !paymentsOverview.data!.features[flag] })
                  }
                />
              </p>
            ))}
            {setFeatures.isError && <p role="alert" className={shared.alert}>{setFeatures.error.message}</p>}
          </>
        )}
      </section>

      <section aria-label="Admins" className={card.card}>
        <h2 className={card.title}>Admins</h2>
        {platformOverview.data && (
          <AdminList
            title="Platform admins"
            admins={platformOverview.data.admins}
            onAdd={(p) => addPlatformAdmin.mutate(p)}
            onRemove={(p) => removePlatformAdmin.mutate(p)}
            adding={addPlatformAdmin.isPending}
            removing={removePlatformAdmin.isPending}
          />
        )}
        {paymentsOverview.data && (
          <AdminList
            title="Payments admins"
            admins={paymentsOverview.data.admins}
            onAdd={(p) => addPaymentsAdmin.mutate(p)}
            onRemove={(p) => removePaymentsAdmin.mutate(p)}
            adding={addPaymentsAdmin.isPending}
            removing={removePaymentsAdmin.isPending}
          />
        )}
        {treasuryConfig.data && (
          <AdminList
            title="Treasury admins"
            admins={treasuryConfig.data.admins}
            onAdd={(p) => setTreasuryAdmins.mutate([...treasuryConfig.data!.admins, p])}
            onRemove={(p) => setTreasuryAdmins.mutate(treasuryConfig.data!.admins.filter((a) => a.toText() !== p.toText()))}
            adding={setTreasuryAdmins.isPending}
            removing={setTreasuryAdmins.isPending}
          />
        )}
      </section>
    </AdminPageShell>
  );
};
