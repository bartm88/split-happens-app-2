import { Transaction } from "../types";

interface TransactionListProps {
  transactions: Transaction[];
  onUndo: () => void;
  onConvert: (transaction: Transaction) => void;
}

export const TransactionList = ({
  transactions,
  onUndo,
  onConvert,
}: TransactionListProps) => {
  // Only show the 5 most recent transactions
  const recentTransactions = transactions.slice(0, 5);

  const isSplit = (transaction: Transaction): boolean => {
    return transaction.creditor !== "Pot" && transaction.debtor === "Pot";
  };

  return (
    <div className="bg-neutral bg-opacity-90 rounded-lg shadow-md border border-neutral">
      <div className="p-4 border-b bg-primary rounded-t-lg">
        <h2 className="text-xl font-semibold text-neutral">Transactions</h2>
      </div>
      <div className="divide-y divide-neutral divide-opacity-20">
        {recentTransactions.length === 0 ? (
          <div className="p-4 text-center text-neutral">
            No transactions yet
          </div>
        ) : (
          recentTransactions.map((transaction, index) => (
            <div
              key={index}
              className="p-4 hover:bg-neutral-hover transition-colors"
            >
              <div className="grid grid-cols-[3fr_1fr_4fr] gap-4 items-center justify-items-start">
                {/* Transaction Details */}
                <div>
                  <div className="flex items-center space-x-2">
                    <span className="font-semibold text-neutral">
                      {transaction.creditor}
                    </span>
                    <span className="text-neutral">→</span>
                    <span className="font-semibold text-neutral">
                      {transaction.debtor}
                    </span>
                  </div>
                  <div className="text-sm text-neutral mt-2">
                    <span className="bg-primary text-primary px-2 py-1 rounded-full text-xs font-medium">
                      {transaction.split}
                    </span>
                  </div>
                  <div className="text-sm text-neutral mt-1">
                    {transaction.date}
                  </div>
                </div>

                {/* Action buttons */}
                <div className="flex gap-4 justify-center">
                  <button
                    onClick={() => onConvert(transaction)}
                    disabled={!isSplit(transaction)}
                    className={`p-2 rounded-md transition-colors w-8 h-8 flex items-center justify-center ${
                      isSplit(transaction)
                        ? "bg-confirmation-hover text-confirmation"
                        : "bg-gray-200 text-gray-400 cursor-not-allowed"
                    }`}
                    title="Convert"
                  >
                    ✓
                  </button>
                  {index === 0 && (
                    <button
                      onClick={onUndo}
                      className="p-2 bg-negative-hover text-negative rounded-md transition-colors w-8 h-8 flex items-center justify-center"
                      title="Undo"
                    >
                      ↩
                    </button>
                  )}
                </div>

                {/* Amounts */}
                <div className="text-right justify-self-end">
                  <div
                    className={`text-lg font-bold px-2 py-1 rounded ${
                      isSplit(transaction)
                        ? "bg-negative text-negative"
                        : "bg-confirmation text-confirmation"
                    }`}
                  >
                    ${transaction.amount.toFixed(2)}
                  </div>
                  <div className="text-sm text-neutral mt-1">
                    Pot: ${transaction.pot_amount.toFixed(2)}
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
