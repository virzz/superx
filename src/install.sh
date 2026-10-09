export SUPERX_TARGET_PATH="/usr/local/sbin/$SUPERX_TARGET_NAME";
test -f "$SUPERX_TARGET_PATH" || exit 1;
test -f $SUPERX_CURRENT_EXE || exit 1;
if [ -f "$SUPERX_TARGET_PATH" ]; then
    chflags nouchg "$SUPERX_TARGET_PATH";
    chflags noschg "$SUPERX_TARGET_PATH";
fi
mkdir -p /usr/local/sbin/;
cp "$SUPERX_CURRENT_EXE" "$SUPERX_TARGET_PATH";
chown root:wheel "$SUPERX_TARGET_PATH";
chmod 4511 "$SUPERX_TARGET_PATH";
chflags uchg "$SUPERX_TARGET_PATH";
chflags schg "$SUPERX_TARGET_PATH";
test -x "$SUPERX_TARGET_PATH" || exit 1;
exit 0;
