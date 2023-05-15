import React from "react";
import {
  createStyles,
  Theme,
  withStyles,
  WithStyles,
} from "@material-ui/core/styles";
import Button from "@material-ui/core/Button";
import Dialog from "@material-ui/core/Dialog";
import MuiDialogTitle from "@material-ui/core/DialogTitle";
import MuiDialogContent from "@material-ui/core/DialogContent";
import MuiDialogActions from "@material-ui/core/DialogActions";
import IconButton from "@material-ui/core/IconButton";
import CloseIcon from "@material-ui/icons/Close";
import Typography from "@material-ui/core/Typography";
// import Divider from "@material-ui/core/Divider";
import Link from "@material-ui/core/Link";

const styles = (theme: Theme) =>
  createStyles({
    root: {
      margin: 0,
      padding: theme.spacing(2),
    },
    closeButton: {
      position: "absolute",
      right: theme.spacing(1),
      top: theme.spacing(1),
      color: theme.palette.grey[500],
    },
  });

export interface DialogTitleProps extends WithStyles<typeof styles> {
  id: string;
  children: React.ReactNode;
  onClose: () => void;
}

const DialogTitle = withStyles(styles)((props: DialogTitleProps) => {
  const { children, classes, onClose, ...other } = props;
  return (
    <MuiDialogTitle disableTypography className={classes.root} {...other}>
      <Typography variant="h6">{children}</Typography>
      {onClose ? (
        <IconButton
          aria-label="close"
          className={classes.closeButton}
          onClick={onClose}
        >
          <CloseIcon />
        </IconButton>
      ) : null}
    </MuiDialogTitle>
  );
});

const DialogContent = withStyles((theme: Theme) => ({
  root: {
    padding: theme.spacing(2),
  },
}))(MuiDialogContent);

const DialogActions = withStyles((theme: Theme) => ({
  root: {
    margin: 0,
    padding: theme.spacing(1),
  },
}))(MuiDialogActions);

export default function AboutDialog({
  open,
  setOpen,
}: {
  open: boolean;
  setOpen: (open: boolean) => void;
}) {
  const handleClose = () => {
    setOpen(false);
  };

  return (
    <div>
      <Dialog
        onClose={handleClose}
        aria-labelledby="about-dialog-title"
        open={open}
      >
        <DialogTitle id="about-dialog-title" onClose={handleClose}>
          Notes / 說明
        </DialogTitle>
        <DialogContent dividers>
          <Typography gutterBottom>
            All the conversion rules including built-in conversion tables and
            CGroups comes from Chinese Wikipedia (MediaWiki) and OpenCC, whose
            communities build and maintain those rulesets. The accuracy of
            conversions is pretty good. But it is still predictable to have some
            wrong conversions.
          </Typography>
          <Typography gutterBottom>
            包括內建轉換表（
            <Link href="https://github.com/wikimedia/mediawiki/blob/master/includes/languages/data/ZhConversion.php#L14">
              1
            </Link>
            、
            <Link href="https://github.com/BYVoid/OpenCC/tree/master/data/dictionary">
              2
            </Link>
            ） 和
            <Link href="https://zh.wikipedia.org/wiki/Wikipedia:%E5%AD%97%E8%A9%9E%E8%BD%89%E6%8F%9B%E8%99%95%E7%90%86/%E5%85%AC%E5%85%B1%E8%BD%89%E6%8F%9B%E7%B5%84">
              公共轉換組
            </Link>
            在內的所有字詞轉換規則均來自中文維基百科（MediaWiki）和OpenCC，並由社群維護。此轉換工具準確性尚可，但仍可預期地會包含錯誤轉換。
          </Typography>
          <Typography gutterBottom>
            {"See also / 另见 "}
            <Link href="https://zh.wikipedia.org/wiki/Help:%E9%AB%98%E7%BA%A7%E5%AD%97%E8%AF%8D%E8%BD%AC%E6%8D%A2%E8%AF%AD%E6%B3%95">
              Help:高级字词转换语法
            </Link>
            {", "}
            <Link href="https://zh.wikipedia.org/wiki/Help:中文维基百科的繁简、地区词处理#转换技术">
              Help:中文维基百科的繁简、地区词处理#转换技术
            </Link>
            .
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button autoFocus onClick={handleClose} color="secondary">
            Ok / 好
          </Button>
        </DialogActions>
      </Dialog>
    </div>
  );
}
