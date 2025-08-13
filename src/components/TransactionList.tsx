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
    <div className="bg-neutral bg-opacity-90 rounded-lg shadow-md">
      <div className="p-4 border-b border-neutral">
        <h2 className="text-xl font-semibold text-neutral">
          Recent Transactions
        </h2>
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
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <div className="flex items-center space-x-2">
                    <span className="font-semibold text-neutral">
                      {transaction.creditor}
                    </span>
                    <span className="text-neutral">→</span>
                    <span className="font-semibold text-neutral">
                      {transaction.debtor}
                    </span>
                  </div>
                  <div className="text-sm text-neutral mt-1">
                    <span className="bg-primary text-primary px-2 py-1 rounded-full text-xs font-medium">
                      {transaction.split}
                    </span>
                    <span className="ml-2">{transaction.date}</span>
                  </div>
                </div>
                <div className="text-right">
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

              {/* Action buttons */}
              <div className="mt-3 flex gap-2">
                {index === 0 && (
                  <button
                    onClick={onUndo}
                    className="px-3 py-1 text-sm bg-negative-hover text-negative rounded-md transition-colors"
                  >
                    Undo
                  </button>
                )}
                {isSplit(transaction) && (
                  <button
                    onClick={() => onConvert(transaction)}
                    className="px-3 py-1 text-sm bg-confirmation-hover text-confirmation rounded-md transition-colors"
                  >
                    Convert
                  </button>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
