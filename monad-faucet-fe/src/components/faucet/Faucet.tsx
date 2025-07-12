import { CisternToken } from "./CisternToken";
import { TokenFiller } from "./TokenFiller";

export const Faucet = () => {
  return (
    <div className="flex w-full max-w-[672px] flex-col gap-10 px-4 py-12">
      <TokenFiller />
      <CisternToken />
    </div>
  );
};
