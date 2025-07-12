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
        "hover:border-yellow-500 flex h-full items-center rounded-lg border border-yellow bg-white/10 px-3 py-2 shadow-sm transition-all duration-150",
        className
      )}
      style={width ? { width } : undefined}
    >
      {type === inputTypes.file ? (
        !imageURL ? (
          <label className="flex w-full cursor-pointer items-center gap-2">
            <span className="text-sm font-semibold text-yellow">
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
              className="h-8 w-8 rounded-full border border-yellow object-cover"
            />
            <span className="text-sm font-semibold text-yellow">
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
              className="ml-2 rounded-full p-1 transition hover:bg-yellow/20 focus:outline-none"
              aria-label="Edit"
            >
              <img src="/EditIcon.svg" alt="Edit" className="h-5 w-5" />
            </button>
          </div>
        )
      ) : (
        <label className="flex w-full items-center gap-2">
          <input
            type={type}
            value={defaultValue}
            onChange={onChange}
            className="w-full border-none bg-transparent font-heldane text-sm font-semibold text-yellow outline-none placeholder:text-yellow/60 focus:outline-none focus:ring-0"
            {...props}
          />
        </label>
      )}
      {imageURL && type === inputTypes.file && (
        <button
          type="button"
          onClick={handleEditClick}
          className="ml-2 rounded-full p-1 transition hover:bg-yellow/20 focus:outline-none"
          aria-label="Edit"
        >
          <img src="/EditIcon.svg" alt="Edit" className="h-5 w-5" />
        </button>
      )}
    </div>
  );
};
