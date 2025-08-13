export interface Transaction {
  creditor: string;
  debtor: string;
  amount: number;
  split: string;
  time: string;
  pot_amount: number;
  date: string;
}

export interface Balance {
  name: string;
  amount: string;
}