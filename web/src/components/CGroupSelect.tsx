import { useState, useEffect, useMemo } from "react";
import {
  createStyles,
  makeStyles,
  useTheme,
  Theme,
} from "@material-ui/core/styles";
import FormControl from "@material-ui/core/FormControl";
import FormGroup from "@material-ui/core/FormGroup";
import FormControlLabel from "@material-ui/core/FormControlLabel";
import FormHelperText from "@material-ui/core/FormHelperText";
import FormLabel from "@material-ui/core/FormLabel";
import InputLabel from "@material-ui/core/InputLabel";
import Select from "@material-ui/core/Select";
import Input from "@material-ui/core/Input";
import MenuItem from "@material-ui/core/MenuItem";
import Chip from "@material-ui/core/Chip";
import Checkbox from "@material-ui/core/Checkbox";
import ListItemText from "@material-ui/core/ListItemText";
import Dialog, { DialogProps } from "@material-ui/core/Dialog";
import DialogActions from "@material-ui/core/DialogActions";
import DialogContent from "@material-ui/core/DialogContent";
import DialogContentText from "@material-ui/core/DialogContentText";
import DialogTitle from "@material-ui/core/DialogTitle";
import Button from "@material-ui/core/Button";
import Grid from "@material-ui/core/Grid";
import Paper from "@material-ui/core/Paper";
import AddIcon from "@material-ui/icons/Add";
import Avatar from "@material-ui/core/Avatar";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";

import CGroupCheckbox from "./CGroupCheckbox";

const useStyles = makeStyles((theme: Theme) =>
  createStyles({
    formControl: {
      margin: theme.spacing(1),
      // minWidth: 120,
      // maxWidth: 300,
      // width: "50%",
    },
    // chips: {
    //   display: "flex",
    //   flexWrap: "wrap",
    // },
    // chip: {
    //   margin: 2,
    // },
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
      margin: theme.spacing(0.5),
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
}: // onChange: handleChange,
{
  cgroups: string[];
  open: boolean;
  onClose: () => void;
  onSelect: (cgroups: string[]) => void;
  selected: string[];
}) {
  // const classes = useStyles();
  // const [selecting, setSelecting] = useState({});
  // const [checkedCGroups, setCheckedCGroups] = useState(cgroups);
  // useEffect(() => { setCheckedCGroups(cgroups) }, [open]);
  // const handleCheck = (event: React.ChangeEvent<HTMLInputElement>) => {
  //   // setChecked({ ...checked, [event.target.name]: event.target.checked });
  // };
  // console.log(checkedCGroups);
  // Note: For performance, Checkboxes are memoized, resulting in captured states to be stale
  //       when get updated. We have to use transition functions for setState.
  //       We maintain a additional somewhat duplicate state here to avoid receive a complex
  //       onSelect callback from the upstream, for ergonomics.
  const [_selected_, setSelected_] = useState([] as string[]);
  const handleCheck = (name: string, checked: boolean) => {
    // console.log([name, checked], cgroups, { ...cgroups, [name]: checked });
    // console.log(selected, name, checked);

    // console.log(Array.from(set));
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
  // this might trigger an extra unnecessary render. but it won't be an actually issue
  useEffect(() => setSelected_(selected), [selected]);
  const handleClear = () => {
    handleSelect([]);
  };
  const handleInvert = () => {
    let set = new Set(selected);
    handleSelect(cgroups.filter((name) => !set.has(name)));
    // console.log(cgroups);
    // handleSelect(
    //   Object.fromEntries(
    //     Object.entries(cgroups).map(([name, checked]) => [
    //       name,
    //       !checked,
    //     ])
    //   )
    // );
  };
  // const handleApply = (event) => {

  // }
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
        {/* </FormControl> */}
      </DialogContent>
      <DialogActions>
        <Grid container direction="row" justifyContent="space-between">
          <Grid item>
            <Button onClick={handleClear} color="primary">
              Clear
            </Button>
            <Button onClick={handleInvert} color="primary">
              Invert
            </Button>
          </Grid>
          <Grid item>
            <Button onClick={handleClose} color="secondary">
              Ok
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
  // const cgroupItems = useMemo(() => {
  //   return cgroups.map((cgroup) => (
  //     <MenuItem
  //       key={cgroup}
  //       value={cgroup} /*  style={getStyles(name, personName, theme)} */
  //     >
  //       <Checkbox checked={selected.indexOf(cgroup) > -1} />
  //       <ListItemText primary={cgroup} />
  //     </MenuItem>
  //   ));
  // }, [cgroups]);
  // console.log(cgroups);
  const handleDelete = (name: string) => {
    const set = new Set(selected);
    set.delete(name);
    handleSelect(Array.from(set));
  };
  console.log("selected", selected);
  return (
    <>
      <FormControl className={classes.formControl}>
        <InputLabel id="cgroups-select-label">CGroups / 公共轉換組</InputLabel>
        <Select
          labelId="cgroups-select-label"
          id="cgroups-select"
          multiple
          value={selected}
          // onChange={}
          open={false}
          onOpen={() => setDialogOpen(true)}
          // input={<Input id="select-multiple-chip" />}
          renderValue={(selected) =>
            selected ? (
              <div className={classes.chips}>
                {(selected as string[]).map((name) => (
                  <Chip
                    key={name}
                    label={name}
                    onDelete={(event) => {
                      event.preventDefault();
                      event.stopPropagation();
                      handleDelete(name);
                    }}
                    className={classes.chip}
                    variant="outlined"
                  />
                ))}
              </div>
            ) : (
              <>"Select CGroups / 選擇公共轉換組"</>
            )
          }
          MenuProps={MenuProps}
          style={{ width: "100%" }}
        >
          {selected.map((name) => (
            <MenuItem key={name} value={name}>
              {name}
            </MenuItem>
          ))}
        </Select>
      </FormControl>
      <FormControl
        className={classes.formControl}
        variant="outlined"
        margin="dense"
        style={{ width: "100%" }}
      >
        <InputLabel id="cgroups-label" htmlFor="ddd-cgroups-select-paper">
          CGroups <br />
          公共轉換組
        </InputLabel>
        {/* <Input id="ddd-cgroups-select-paper" innerHtml="<he>" /> */}
      </FormControl>
      {/* <Button onClick={() => setDialogOpen(true)}> CGroups </Button> */}
      <Paper
        id="cgroups-select-paper"
        component="ul"
        className={classes.chips}
        variant="outlined"
      >
        <Grid container direction="row" justifyContent="center">
          <Grid item alignItems="center">
            {/* <Typography> */}
            <FormLabel component="legend">
              <Grid container direction="column" justifyContent="center">
                <Grid item>CGroups</Grid>

                <Grid item>
                  {/* <Typography> */}
                  公共轉換組
                  {/* </Typography> */}
                </Grid>
              </Grid>
            </FormLabel>
            {/* </Typography> */}
          </Grid>
          <Grid item style={{ flexGrow: 1, flexBasis: 1 }}>
            <Box component="ul" className={classes.chips}>
              {selected.map((name) => (
                <li key={name}>
                  <Chip
                    // icon={icon}
                    variant="outlined"
                    label={name}
                    onDelete={() => handleDelete(name)}
                    className={classes.chip}
                  />
                </li>
              ))}
              <li key="add more">
                <Chip
                  // avatar={<Avatar><AddIcon /></Avatar>}
                  // label={selected.length > 0 ? "Add more" : "Add CGroups"}
                  clickable
                  color="primary"
                  onClick={() => setDialogOpen(true)}
                  onDelete={() => setDialogOpen(true)}
                  deleteIcon={
                    <Avatar>
                      <AddIcon />
                    </Avatar>
                  }
                  variant="outlined"
                />
                {/* <Avatar><AddIcon /></Avatar> */}
              </li>
            </Box>
          </Grid>
        </Grid>
      </Paper>
      <CGroupDialog
        cgroups={cgroups}
        selected={selected}
        onSelect={handleSelect}
        // cgroups={Object.fromEntries(
        //   cgroups.map((name) => [name, selected.indexOf(name) > -1])
        // )}
        // onSelect={(cgroups) =>
        //   {
        //     console.log("k", cgroups)
        //     handleSelect(
        //     Object.entries(cgroups)
        //       .filter(([_name, selected]) => selected)
        //       .map(([name, _selected]) => name)
        //   )}
        // }
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
      />
    </>
  );
}
