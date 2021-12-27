import { useState, useEffect } from "react";
import {
  createStyles,
  makeStyles,
  // useTheme,
  Theme,
} from "@material-ui/core/styles";
import FormControl from "@material-ui/core/FormControl";
import FormHelperText from "@material-ui/core/FormHelperText";
import InputLabel from "@material-ui/core/InputLabel";
import Select from "@material-ui/core/Select";
import MenuItem from "@material-ui/core/MenuItem";
import Chip from "@material-ui/core/Chip";
import Dialog /*, { DialogProps }*/ from "@material-ui/core/Dialog";
import DialogActions from "@material-ui/core/DialogActions";
import DialogContent from "@material-ui/core/DialogContent";
import DialogTitle from "@material-ui/core/DialogTitle";
import Button from "@material-ui/core/Button";
import Grid from "@material-ui/core/Grid";

import CGroupCheckbox from "./CGroupCheckbox";

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    formControl: {
      margin: theme.spacing(1),
      // minWidth: 120,
      // maxWidth: 300,
      // width: "50%",
    },
    noLabel: {
      marginTop: theme.spacing(3),
    },
    chips: {
      display: "flex",
      justifyContent: "center",
      flexWrap: "wrap",
      listStyle: "none",
      padding: theme.spacing(0.5),
      margin: 0,
    },
    chip: {
      margin: theme.spacing(0.3),
      cursor: "pointer",
    },
  })
);

const ITEM_HEIGHT = 48;
const ITEM_PADDING_TOP = 8;
const MenuProps = {
  PaperProps: {
    style: {
      maxHeight: ITEM_HEIGHT * 4.5 + ITEM_PADDING_TOP,
      width: 250,
    },
  },
  keepMounted: true,
};

function CGroupDialog({
  cgroups,
  open,
  onClose: handleClose,
  onSelect: handleSelect,
  selected,
}: {
  cgroups: string[];
  open: boolean;
  onClose: () => void;
  onSelect: (cgroups: string[]) => void;
  selected: string[];
}) {
  // Note: For performance, Checkboxes are memoized, resulting in captured states to be stale
  //       when get updated. We have to use transition functions for setState.
  //       We maintain a additional somewhat duplicate state here to avoid receive a complex
  //       onSelect callback from the upstream, for ergonomics.
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_selected_, setSelected_] = useState([] as string[]);
  const handleCheck = (name: string, checked: boolean) => {
    setSelected_((prev: string[]) => {
      let set = new Set(prev);
      if (checked) {
        set.add(name);
      } else {
        set.delete(name);
      }
      handleSelect(Array.from(set));
      return Array.from(set);
    });
  };
  // this might trigger an extra unnecessary render. but it won't cause an actually trouble
  useEffect(() => setSelected_(selected), [selected]);
  const handleClear = () => {
    handleSelect([]);
  };
  const handleInvert = () => {
    let set = new Set(selected);
    handleSelect(cgroups.filter((name) => !set.has(name)));
  };
  return (
    <Dialog
      open={open}
      onClose={handleClose}
      scroll="paper"
      aria-labelledby="cgroups-dialog-title"
      aria-describedby="cgroups-dialog-description"
      keepMounted
    >
      <DialogTitle id="cgroups-dialog-title">CGroups / 公共轉換組</DialogTitle>
      <DialogContent dividers>
        {cgroups.map((name) => (
          <CGroupCheckbox
            key={name}
            name={name}
            checked={selected.indexOf(name) > -1}
            onCheck={handleCheck}
          />
          // <FormControlLabel key={name} control={<input type="checkbox" id={name} name={name} value={name} />} label={name} />
        ))}
        <FormHelperText>Press Ctrl + F to search</FormHelperText>
      </DialogContent>
      <DialogActions>
        <Grid container direction="row" justifyContent="space-between">
          <Grid item>
            <Button onClick={handleClear} color="primary">
              Clear / 清空
            </Button>
            <Button onClick={handleInvert} color="primary">
              Invert / 反選
            </Button>
          </Grid>
          <Grid item>
            <Button onClick={handleClose} color="secondary">
              Ok / 好
            </Button>
          </Grid>
        </Grid>
      </DialogActions>
    </Dialog>
  );
}

export default function CGroupSelect({
  cgroups,
  selected,
  onSelect: handleSelect,
}: {
  cgroups: string[];
  selected: string[];
  onSelect: (selected: string[]) => void;
}) {
  const classes = useStyles();
  const [dialogOpen, setDialogOpen] = useState(false);
  // const handleDelete = (name: string) => {
  //   const set = new Set(selected);
  //   set.delete(name);
  //   handleSelect(Array.from(set));
  // };
  return (
    <>
      <FormControl className={classes.formControl}>
        <InputLabel id="cgroups-select-label">CGroups / 公共轉換組</InputLabel>
        <Select
          labelId="cgroups-select-label"
          id="cgroups-select"
          multiple
          value={selected.length > 0 ? selected : ["placeholder"]}
          open={false}
          onOpen={() => setDialogOpen(true)}
          style={{ width: "100%" }}
          fullWidth={true}
          // input={<Input id="select-multiple-chip" />}
          renderValue={(selected) => (
            <div className={classes.chips}>
              {(selected as string[]).length === 0 ||
              (selected as string[])[0] === "placeholder" ? (
                <Chip
                  key="add more"
                  label="Select CGroups / 選擇公共轉換組 ..."
                  color="primary"
                  className={classes.chip}
                  variant="outlined"
                />
              ) : (
                (selected as string[]).map((name) => (
                  <Chip
                    key={name}
                    label={name}
                    // the ondelete event seems to be shadowed anyway
                    // onDelete={(event) => {
                    //   event.preventDefault();
                    //   event.stopPropagation();
                    //   handleDelete(name);
                    // }}
                    className={classes.chip}
                    variant="outlined"
                  />
                ))
              )}
            </div>
          )}
          MenuProps={MenuProps}
        >
          {selected.map((name) => (
            <MenuItem key={name} value={name}>
              {name}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <CGroupDialog
        cgroups={cgroups}
        selected={selected}
        onSelect={handleSelect}
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
      />
    </>
  );
}
