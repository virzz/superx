# SuperX

SuperX is a tool for macOS to authenticate users with Touch ID or password.

## Install

```bash
sudo chown root:root superx
sudo chown root:wheel superx
sudo chmod 4511 superx
sudo chflags uchg superx # User不可改
sudo chflags schg superx # SuperUser不可改
```

## Usage

```bash
./superx
```

## License

MIT