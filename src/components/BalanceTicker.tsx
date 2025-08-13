import { Balance } from "../types";

interface BalanceTickerProps {
  balances: Balance[];
}

export const BalanceTicker = ({ balances }: BalanceTickerProps) => {
  const formatBalance = (balance: Balance) => {
    const amount = parseFloat(balance.amount);
    const isPositive = amount >= 0;
    const isPot = balance.name === "Pot";

    if (isPot) {
      return `POT: $${Math.abs(amount).toFixed(2)}`;
    }

    const prefix = isPositive ? "" : "-";
    return `${balance.name}: ${prefix}$${Math.abs(amount).toFixed(2)}`;
  };

  const getBalanceClass = (balance: Balance) => {
    const amount = parseFloat(balance.amount);
    const isPot = balance.name === "Pot";

    if (isPot) return "";
    return amount >= 0 ? "positive-balance" : "negative-balance";
  };

  // Create the ticker text and replicate it 3 times for continuous scrolling
  const createTickerContent = () => {
    if (balances.length === 0) {
      return "<span>Loading balances...</span>";
    }

    const singleText = balances
      .map(
        (balance) =>
          `<span class="${getBalanceClass(balance)}">${formatBalance(
            balance
          )}</span>`
      )
      .join(" ");

    // Replicate the text 3 times with separators
    return `${singleText} ${singleText} ${singleText}`;
  };

  return (
    <div className="balance-ticker">
      <div
        className="ticker-content"
        dangerouslySetInnerHTML={{ __html: createTickerContent() }}
      />
    </div>
  );
};
