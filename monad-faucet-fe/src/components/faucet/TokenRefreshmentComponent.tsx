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

  const handleHere = (txHash: string) => {
    window.open(API().explorer + `${txHash}`, "_blank");
  };

  return !txHash ? (
    <>
      <div className="flex w-full items-center justify-start gap-4">
        <h1 className="text-base sm:text-2xl">
          No refreshment for now, come back{" "}
          {nextAccess
            ? ` after ${convertToLocalTime(nextAccess)}`
            : "later......"}
        </h1>
      </div>
      <div className="flex w-full items-baseline justify-start gap-2 sm:gap-4">
        <h1 className="text-sm sm:text-2xl">Until then go visit a beauty </h1>
        <a href="https://new.garden.finance/" target="_blank">
          <img
            src="/Here.png"
            alt="Redirecter to Garden"
            className="h-[15px] w-fit cursor-pointer sm:h-[25px]"
          />
        </a>
        <h1 className="text-sm sm:text-2xl">lovely!</h1>
      </div>

      <div className="mt-3 flex w-full items-center justify-end gap-6 sm:mb-3 sm:mt-12">
        <Button className="w-fit" handleClick={onClose}>
          Okay,i'll come for the token later
        </Button>
      </div>
    </>
  ) : (
    <>
      <div className="flex w-full flex-wrap items-center justify-start gap-4">
        {magnification > 1 ? (
          <>
            <div className="flex flex-wrap gap-1.5 text-sm sm:text-2xl">
              <h1 className="">The</h1>
              <h1 className="">refreshment</h1>
              <h1 className="">token</h1>
              <h1 className="">you</h1>
              <h1 className="">requested</h1>
              <h1 className="">is</h1>
              <h1 className="">on</h1>
              <h1 className="">it's</h1>
              <h1 className="">way,</h1>
              <h1 className="">with</h1>
              {magnification == 20 ? (
                <img
                  src="/20Magnification.webp"
                  alt=""
                  className="w-[42px] -translate-y-3.5"
                />
              ) : (
                <img
                  src="/10Magnification.webp"
                  alt=""
                  className="w-[42px] -translate-y-3.5"
                />
              )}
              <h1 className="">boost</h1>
              <h1 className="text-sm sm:text-2xl">
                (
                {formatUnits(
                  amount ? BigInt(amount) : BigInt(0),
                  dripToken?.decimals ?? 0
                ) +
                  " " +
                  dripToken?.symbol}{" "}
                )
              </h1>
              <h1 className="text-sm sm:text-2xl">, wow!</h1>
            </div>
          </>
        ) : (
          <h1 className="text-sm sm:text-2xl">
            The refreshment (
            {formatUnits(
              amount ? BigInt(amount) : BigInt(0),
              dripToken?.decimals ?? 0
            ) +
              " " +
              dripToken?.symbol}{" "}
            ) token you requested is on it's way, wow!
          </h1>
        )}
      </div>
      <div className="flex w-full items-baseline justify-start gap-2 sm:gap-4">
        <h1 className="text-sm sm:text-2xl">check</h1>
        {txHash && (
          <img
            src="/Here.png"
            alt="Redirector to Monad explorer"
            className="h-[15px] w-fit sm:h-[25px]"
            onClick={() => handleHere(txHash)}
          />
        )}
        <h1 className="text-sm sm:text-2xl">how it&apos;s doing.</h1>
      </div>

      <div className="mt-3 flex w-full items-center justify-end gap-6 sm:mb-3 sm:mt-12">
        <Button className="w-fit" handleClick={onClose}>
          wow,thank you for the refreshment
        </Button>
      </div>
    </>
  );
};
