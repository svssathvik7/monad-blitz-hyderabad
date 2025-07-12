import { handleQuenchTokens, isValidEVMAddress } from "../../utils/utils";
import { modalNames, modalStore } from "../../store/ModalStore";
import { useAppStore } from "../../store/store";
import { Token, useTokenListStore } from "../../store/TokenListStore";
import CustomDropdown from "./CustomDropdown";
import { useState } from "react";
import { Button } from "../ui/Button";

export const TokenFiller = () => {
  const [token, setToken] = useState<Token>();
  const [addressInput, setAddressInput] = useState("");

  const { setOpenModal } = modalStore();
  const {
    isError,
    isLoading,
    setSuccess,
    setLoading,
    setError,
    setTxHash,
    setAmount,
    setNextAccess,
    setDripToken,
    setMagnification,
  } = useAppStore();
  const { tokens } = useTokenListStore();

  const disabled = !token || !addressInput || !isValidEVMAddress(addressInput);
  const buttonText = !token
    ? "Select Token"
    : !addressInput
      ? "Enter Wallet Address"
      : isValidEVMAddress(addressInput)
        ? "Request Testnet Tokens"
        : "Invalid Address";

  const handleClick = async () => {
    if (!token || !addressInput) return;

    const data = await handleQuenchTokens(
      addressInput,
      token,
      setLoading,
      setError
    );

    if (data.error && isError) {
      setError(true);
      setOpenModal(modalNames.tokenRefreshments);
    }

    if (!isLoading) {
      setSuccess(true);
      setOpenModal(modalNames.tokenRefreshments);

      const txHash = data?.data?.tx_hash ?? null;
      const amount = data?.data?.amount ?? null;
      const nextAccess = data?.error?.next_access ?? null;
      const magnification = data?.data?.magnification ?? 1;
      setTxHash(txHash);
      setAmount(amount);
      setNextAccess(nextAccess);
      setDripToken(token);
      setMagnification(magnification);
    }
  };

  return (
    <div className="to-blue-50 z-20 mx-auto flex min-h-[400px] max-w-md flex-col items-center justify-center rounded-2xl border border-gray-100 bg-gradient-to-br from-white via-gray-50 px-8 py-10 shadow-2xl">
      <div className="mb-6 flex flex-col items-center">
        <img
          src="https://cdn.prod.website-files.com/667c57e6f9254a4b6d914440/66956efeb01de969bd7b8abc_Logo%20Horizontal.svg"
          alt="Monad Logo"
          className="mb-2 w-[75%]"
        />

        <p className="mt-2 text-center text-base text-gray-500">
          Instantly get free testnet tokens for your wallet.
        </p>
      </div>
      <div className="mb-5 flex w-full flex-col items-center justify-center">
        <CustomDropdown
          tokens={tokens}
          selectedToken={token}
          onSelect={setToken}
        />
      </div>
      <div className="mb-6 w-full">
        <label className="mb-2 block text-sm font-semibold text-gray-700">
          Wallet Address
        </label>
        <input
          type="text"
          className="focus:ring-blue-500 w-full rounded-xl border border-gray-200 bg-gray-50 px-4 py-2 text-base text-gray-700 transition focus:outline-none focus:ring-2"
          placeholder="0x..."
          value={addressInput}
          onChange={(e) => setAddressInput(e.target.value)}
        />
        {!isValidEVMAddress(addressInput) && addressInput && (
          <span className="mt-1 block text-xs text-red-500">
            Please enter a valid EVM address.
          </span>
        )}
      </div>
      <Button
        handleClick={handleClick}
        disabled={disabled}
        className={`w-full rounded-xl py-2 font-semibold text-white transition ${
          disabled
            ? "cursor-not-allowed bg-gray-300"
            : "bg-blue-600 hover:bg-blue-700 shadow"
        }`}
      >
        {buttonText}
      </Button>
    </div>
  );
};
