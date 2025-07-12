import React, { FC, useRef } from "react";
import { cn } from "../../utils/utils";

export const inputTypes = {
  text: "text",
  number: "number",
  file: "file",
} as const;

export type InputType = keyof typeof inputTypes;

type InputBoxProps = {
  type?: InputType;
  defaultValue?: string;
  className?: string;
  imageURL?: string;
  width?: string;
  placeholder?: string;
  accept?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
};

export const InputBox: FC<InputBoxProps> = ({
  type = inputTypes.text,
  defaultValue,
  imageURL,
  className,
  width,
  accept,
  onChange,
  placeholder,
  ...props
}) => {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleEditClick = () => {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  };

  return (
    <div
      className={cn(
        "border-yellow flex h-full items-center rounded-lg border bg-white/10 px-3 py-2 shadow-sm transition-all duration-150 hover:border-yellow-500",
        className
      )}
      style={width ? { width } : undefined}
    >
      {type === inputTypes.file ? (
        !imageURL ? (
          <label className="flex w-full cursor-pointer items-center gap-2">
            <span className="text-yellow text-sm font-semibold">
              Choose file
            </span>
            <input
              ref={fileInputRef}
              type="file"
              accept={accept}
              onChange={onChange}
              className="hidden outline-none focus:outline-none"
              {...props}
            />
          </label>
        ) : (
          <div className="flex w-full items-center gap-2">
            <img
              src={imageURL}
              alt="Uploaded Preview"
              className="border-yellow h-8 w-8 rounded-full border object-cover"
            />
            <span className="text-yellow text-sm font-semibold">
              {defaultValue && defaultValue.length <= 12
                ? defaultValue
                : `${defaultValue?.slice(0, 12)}...`}
            </span>
            <input
              ref={fileInputRef}
              type="file"
              accept={accept}
              onChange={onChange}
              className="hidden outline-none focus:outline-none"
              {...props}
            />
            <button
              type="button"
              onClick={handleEditClick}
              className="hover:bg-yellow/20 ml-2 rounded-full p-1 transition focus:outline-none"
              aria-label="Edit"
            ></button>
          </div>
        )
      ) : (
        <label className="flex w-full items-center gap-2">
          <input
            type={type}
            value={defaultValue}
            onChange={onChange}
            className="font-heldane text-yellow placeholder:text-yellow/60 w-full border-none bg-transparent text-sm font-semibold outline-none focus:outline-none focus:ring-0"
            {...props}
          />
        </label>
      )}
      {imageURL && type === inputTypes.file && (
        <button
          type="button"
          onClick={handleEditClick}
          className="hover:bg-yellow/20 ml-2 rounded-full p-1 transition focus:outline-none"
          aria-label="Edit"
        >
          <img src="/EditIcon.svg" alt="Edit" className="h-5 w-5" />
        </button>
      )}
    </div>
  );
};
