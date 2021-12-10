import React, { useMemo, MouseEvent } from "react";
import { makeStyles } from "@material-ui/core/styles";
import Box from "@material-ui/core/Box";
import Fab from "@material-ui/core/Fab";
import Tooltip from "@material-ui/core/Tooltip";
import Popover from "@material-ui/core/Popover";
import Badge from "@material-ui/core/Badge";
import WarningIcon from "@material-ui/icons/Warning";
import Typography from "@material-ui/core/Typography";
import Paper from "@material-ui/core/Paper";
import orange from "@material-ui/core/colors/orange";

import { countLines } from "../utils";

const useStyles = makeStyles((theme) => ({
  fab: {
    position: "absolute",
    right: theme.spacing(3),
    bottom: theme.spacing(5),
    color: orange[500],
    backgroundColor: "white",
    borderColor: theme.palette.primary.light,
    // "&:hover": {
    //   backgroundColor: "white",
    // }
  },
  popoverContent: {
    margin: theme.spacing(1),
  },
  popoverTitle: {
    textAlign: "center",
  },
  popoverBody: {
    padding: theme.spacing(1),
    margin: 0,
    minWidth: "12em",
  },
}));

export default function WarningFab({ invalidLines }: { invalidLines: string }) {
  const classes = useStyles();

  const [anchorEl, setAnchorEl] = React.useState(null as any);
  const handleOpen = (event: MouseEvent) => {
    setAnchorEl(event.currentTarget);
  };
  const handleClose = () => {
    setAnchorEl(null);
  };
  const open = Boolean(anchorEl);
  const id = open ? "invalid-lines-popover" : undefined;

  const invalidCount = useMemo(() => countLines(invalidLines), [invalidLines]);

  return invalidCount > 0 ? (
    <>
      <Tooltip title={invalidCount + " invalid lines"}>
        <Fab
          size="small"
          className={classes.fab}
          aria-label="Show warnings"
          onClick={handleOpen}
        >
          <Badge badgeContent={invalidCount} color="secondary">
            <WarningIcon />
          </Badge>
        </Fab>
      </Tooltip>
      <Popover
        id={id}
        open={open}
        anchorEl={anchorEl}
        onClose={handleClose}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "center",
        }}
        transformOrigin={{
          vertical: "top",
          horizontal: "center",
        }}
      >
        <Box className={classes.popoverContent}>
          <Typography variant="h6" className={classes.popoverTitle}>
            Invalid lines
          </Typography>
          <Paper variant="outlined" square>
            <pre className={classes.popoverBody}>
              <code>{invalidLines}</code>
            </pre>
          </Paper>
        </Box>
      </Popover>
    </>
  ) : (
    <></>
  );
}
