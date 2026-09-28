import type { ButtonHTMLAttributes } from 'react';
import { Link, type LinkProps } from 'react-router-dom';
import styles from './Button.module.css';

export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost';
export type ButtonSize = 'sm' | 'md';

interface Look {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export const buttonClass = ({ variant = 'secondary', size = 'md' }: Look, extra?: string) =>
  [styles.button, styles[variant], styles[size], extra].filter(Boolean).join(' ');

export function Button({
  variant,
  size,
  busy,
  className,
  type = 'button',
  disabled,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & Look & { busy?: boolean }) {
  return (
    <button
      type={type}
      className={buttonClass({ variant, size }, className)}
      disabled={disabled || busy}
      aria-busy={busy || undefined}
      {...rest}
    />
  );
}

export function ButtonLink({ variant, size, className, ...rest }: LinkProps & Look) {
  return <Link className={buttonClass({ variant, size }, className)} {...rest} />;
}
