import { FC, ReactNode } from "react";
import { Navbar } from "./components/common/Navbar";

type LayoutProps = {
  children: ReactNode;
};

export const Layout: FC<LayoutProps> = ({ children }) => {
  return (
    <div className="max-w-screen h-full bg-purple-400 font-heldane">
      <Navbar />
      <div className="max-w-screen flex w-screen justify-center overflow-hidden font-heldane text-lg text-yellow sm:h-full">
        {children}
      </div>
    </div>
  );
};
