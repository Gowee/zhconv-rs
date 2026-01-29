import React from "react";
import FormControlLabel from "@mui/material/FormControlLabel";
import Checkbox from "@mui/material/Checkbox";

type Props = {
  name: string;
  checked: boolean;
  onCheck?: (name: string, checked: boolean) => void;
};

function CGroupCheckbox({ name, checked, onCheck: handleCheck }: Props) {
  return (
    <FormControlLabel
      key={name}
      control={
        <Checkbox
          checked={checked}
          onChange={(event) =>
            handleCheck && handleCheck(event.target.name, event.target.checked)
          }
          name={name}
        />
      }
      label={name}
    />
  );
}

function areEqual(prevProps: Props, nextProps: Props) {
  return (
    prevProps.name === nextProps.name && prevProps.checked === nextProps.checked
  );
}

// export default CGroupCheckbox;
export default React.memo(CGroupCheckbox, areEqual);
