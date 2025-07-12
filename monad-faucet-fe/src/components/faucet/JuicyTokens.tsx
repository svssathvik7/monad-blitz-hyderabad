import { Button } from "../ui/Button";
import { API } from "../../constants/api";

export const JuicyTokens = () => {
  const handleTokenListClick = () => {
    window.open(API().tokens, "_blank", "noopener,noreferrer");
  };
  return (
    <div className="z-20 mx-auto flex max-w-md flex-col items-center justify-center rounded-2xl border border-gray-100 bg-gradient-to-br from-white via-gray-50 to-blue-50 px-8 py-10 shadow-2xl">
      <h1 className="w-full text-center text-xl leading-[20px] sm:text-2xl">
        See the token list here
      </h1>
      <div className="flex w-full items-center justify-end">
        <Button handleClick={handleTokenListClick}>Token List</Button>
      </div>
    </div>
  );
};
