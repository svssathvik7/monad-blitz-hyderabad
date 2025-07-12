import { useState } from "react";

export const Treasure = () => {
  const [isTreasureOpen, setIsTreasureOpen] = useState(false);

  const handleTreasureClick = () => setIsTreasureOpen(!isTreasureOpen);

  const handleFreakin = () =>
    window.open("https://new.garden.finance/", "_blank", "noopener,noreferrer");

  return (
    <div className="fixed bottom-4 left-[3%] z-50 sm:bottom-[16%]">
      <div className="relative">
        {isTreasureOpen && (
          <>
            <h1 className="flex items-center gap-2 text-base sm:text-2xl">
              try out
              <img src="/GardenLogo.svg" alt="" className="h-7 sm:h-full" />
              and
            </h1>
            <div className="flex w-full flex-wrap space-y-3">
              <h1 className="inline-block w-full flex-wrap text-base leading-[22px] sm:text-2xl sm:leading-[33.6px]">
                <span className="mb-2 flex gap-2">
                  get a{" "}
                  <img
                    onClick={handleFreakin}
                    src="/Freakin.png"
                    alt=""
                    className="h-[15px] sm:h-[27px]"
                  />
                  10x
                </span>
                <span>more testnet tokens</span>
                <br />
                <br />
                absolute treasure, just sitting there
              </h1>
            </div>
          </>
        )}
        <button className=" " onClick={handleTreasureClick}>
          {isTreasureOpen ? (
            <img
              src="/TreasureOpen.png"
              alt=""
              className="mt-5 shadow-yellow"
            />
          ) : (
            <img src="/Treasure.png" alt="" className="mt-5 shadow-blue" />
          )}
        </button>
      </div>
    </div>
  );
};
