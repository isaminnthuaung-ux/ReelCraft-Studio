import { useState } from 'react';
import type { InputHTMLAttributes } from 'react';

type Props = Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'value' | 'onChange' | 'min' | 'max'> & {
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
};

export default function NumberInput({ value, min, max, onChange, onFocus, onBlur, ...props }: Props) {
  const [draft, setDraft] = useState<string | null>(null);
  const clamp = (number: number) => Math.min(max, Math.max(min, number));

  return <input {...props} type="number" min={min} max={max} value={draft ?? value}
    onFocus={event => { setDraft(event.currentTarget.value); onFocus?.(event); }}
    onChange={event => {
      const input = event.currentTarget.value;
      setDraft(input);
      // Keep incomplete edits local; saved settings always contain valid numbers.
      if (input !== '' && Number.isFinite(event.currentTarget.valueAsNumber)) {
        onChange(clamp(event.currentTarget.valueAsNumber));
      }
    }}
    onBlur={event => { setDraft(null); onBlur?.(event); }}
  />;
}
