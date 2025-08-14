import { Transaction } from "../types";

interface TransactionListProps {
  transactions: Transaction[];
  onUndo: () => void;
  onConvert: (transaction: Transaction) => void;
  onRefresh: () => void;
  refreshing: boolean;
}

export const TransactionList = ({
  transactions,
  onUndo,
  onConvert,
  onRefresh,
  refreshing,
}: TransactionListProps) => {
  // Only show the 5 most recent transactions
  const recentTransactions = transactions.slice(0, 5);

  const isSplit = (transaction: Transaction): boolean => {
    return transaction.creditor !== "Pot" && transaction.debtor === "Pot";
  };

  return (
    <div className="bg-neutral bg-opacity-90 rounded-lg shadow-md border border-neutral">
      <div className="p-4 border-b bg-primary rounded-t-lg flex justify-between items-center">
        <h2 className="text-xl font-semibold text-neutral">Transactions</h2>
        <button
          onClick={onRefresh}
          disabled={refreshing}
          className="bg-secondary-hover text-secondary p-2 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed w-8 h-8 flex items-center justify-center"
          title="Refresh Data"
        >
          <svg
            className={`w-3 h-3 ${refreshing ? "animate-spin" : ""}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99"
            />
          </svg>
        </button>
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
              <div className="grid grid-cols-[3fr_1fr_4fr] gap-4 items-center">
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
                <div className="flex gap-4">
                  <button
                    onClick={() => onConvert(transaction)}
                    disabled={!isSplit(transaction) || refreshing}
                    className={`p-2 rounded-md transition-colors w-8 h-8 flex items-center justify-center ${
                      isSplit(transaction) && !refreshing
                        ? "bg-confirmation-hover text-confirmation"
                        : "bg-gray-200 text-gray-400 cursor-not-allowed"
                    }`}
                    title="Convert"
                  >
                    <svg
                      className="w-3 h-3"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                      xmlns="http://www.w3.org/2000/svg"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M5 13l4 4L19 7"
                      />
                    </svg>
                  </button>
                  {index === 0 ? (
                    <button
                      onClick={onUndo}
                      disabled={refreshing}
                      className={`p-2 rounded-md transition-colors w-8 h-8 flex items-center justify-center ${
                        refreshing
                          ? "bg-gray-200 text-gray-400 cursor-not-allowed"
                          : "bg-negative-hover text-negative"
                      }`}
                      title="Undo"
                    >
                      <svg
                        className="w-3 h-3"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M9 15L3 9m0 0l6-6M3 9h12a6 6 0 010 12h-3"
                        />
                      </svg>
                    </button>
                  ) : (
                    <div className="w-8 h-8"></div>
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
