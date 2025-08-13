import { Balance } from '../types';

interface BalanceCardProps {
  balance: Balance;
}

export const BalanceCard = ({ balance }: BalanceCardProps) => {
  const amount = parseFloat(balance.amount);
  const isPositive = amount >= 0;
  const isPot = balance.name === 'Pot';

  return (
    <div className={`bg-neutral bg-opacity-90 rounded-lg shadow-md p-4 border-l-4 ${
      isPot ? 'border-primary' : isPositive ? 'border-confirmation' : 'border-negative'
    }`}>
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold text-neutral">{balance.name}</h3>
        <span className={`text-xl font-bold ${
          isPot ? 'text-primary' : isPositive ? 'text-confirmation' : 'text-negative'
        }`}>
          ${Math.abs(amount).toFixed(2)}
        </span>
      </div>
      {!isPot && (
        <p className="text-sm text-neutral opacity-80 mt-1">
          {isPositive ? 'Owed to you' : 'You owe'}
        </p>
      )}
    </div>
  );
};