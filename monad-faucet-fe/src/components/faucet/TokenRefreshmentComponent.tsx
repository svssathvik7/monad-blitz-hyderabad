import { FC } from "react";
import { Button } from "../ui/Button";
import { useAppStore } from "../../store/store";
import { API } from "../../constants/api";
import { convertToLocalTime } from "../../utils/utils";
import { formatUnits } from "viem";

type TokenRefreshmentProps = {
  onClose: () => void;
};

export const TokenRefreshmentComponent: FC<TokenRefreshmentProps> = ({
  onClose,
}) => {
  const { txHash, amount, nextAccess, dripToken, magnification } =
    useAppStore();

  const handleExplorerClick = (hash: string) => {
    window.open(`${API().explorer}${hash}`, "_blank");
  };

  const formattedAmount =
    formatUnits(amount ? BigInt(amount) : BigInt(0), dripToken?.decimals ?? 0) +
    " " +
    (dripToken?.symbol ?? "");

  return (
    <div className="mx-auto w-full max-w-xl rounded-lg bg-white p-6 shadow-md">
      {!txHash ? (
        <>
          <div className="mb-4">
            <h2 className="text-lg font-semibold text-gray-800 sm:text-2xl">
              No tokens available right now.
            </h2>
            <p className="mt-2 text-gray-600">
              {nextAccess
                ? `You can request again after ${convertToLocalTime(nextAccess)}.`
                : "Please check back later."}
            </p>
          </div>
          <div className="flex justify-end">
            <Button className="w-fit" handleClick={onClose}>
              Okay
            </Button>
          </div>
        </>
      ) : (
        <>
          <div className="mb-4 flex flex-col gap-2">
            <h2 className="text-lg font-semibold text-gray-800 sm:text-2xl">
              Your tokens are on your way!
            </h2>
            <div className="flex items-center gap-2">
              {magnification > 1 && (
                <img
                  src={
                    magnification === 20
                      ? "/20Magnification.webp"
                      : "/10Magnification.webp"
                  }
                  alt="Magnification"
                  className="h-10 w-10"
                />
              )}
              <span className="text-gray-700">
                Amount: <span className="font-bold">{formattedAmount}</span>
                {magnification > 1 && (
                  <span className="ml-2 text-green-600">Boosted!</span>
                )}
              </span>
            </div>
          </div>
          <div className="mb-6 flex items-center gap-2">
            <span className="text-gray-700">Check transaction status:</span>
            <Button handleClick={() => handleExplorerClick(txHash)}>
              View on Explorer
            </Button>
          </div>
          <div className="flex justify-end">
            <Button className="w-fit" handleClick={onClose}>
              Thank you!
            </Button>
          </div>
        </>
      )}
    </div>
  );
};
