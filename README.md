# HabitVault 🏆

A secure, privacy-focused habit tracking application built with Rust and GTK4. Track your daily habits with end-to-end encryption and an intuitive calendar interface.

<img width="3802" height="2076" alt="swappy-20250712_143535" src="https://github.com/user-attachments/assets/e610751e-af74-486f-a9d6-20842fe0d2b8" />

## Features

### 🔐 Security & Privacy
- **End-to-end encryption** using AES-256-GCM with Argon2 password hashing
- **Local data storage** - your habits never leave your device
- **Secure password management** with encrypted backup/restore functionality
- **Password change** capability with data re-encryption

### 📅 Habit Management
- **Create and track habits** with customizable names and descriptions
- **Interactive calendar view** for each habit showing completion history
- **Streak tracking** with visual indicators (😞 for 0-2 days, 😊 for 3-6 days, 🔥 for 7+ days)
- **One-click completion** marking for today's habits
- **Edit and delete** habits with confirmation dialogs

### 🎨 User Interface
- **Modern GTK4 interface** with libadwaita styling
- **Dark/light theme** toggle in settings
- **Responsive design** that works well on various screen sizes
- **Intuitive navigation** with expandable calendar views
- **Toast notifications** for user feedback

### 💾 Data Management
- **Encrypted backup export** - create password-protected backup files
- **Secure backup import** - restore from encrypted backup files
- **Data persistence** across application restarts
- **Automatic data saving** after each habit interaction

## Installation

### Prerequisites
- Rust (latest stable version)
- GTK4 development libraries
- libadwaita development libraries

### Building from Source

```bash
# Clone the repository
git clone https://github.com/codedsleep/habit-vault.git
cd habit-vault

# Build and run
cargo build --release
cargo run
```

### AppImage
A pre-built AppImage is available in the releases section for easy installation on most Linux distributions.

## Usage

### First Launch
1. **Set up password**: On first launch, you'll be prompted to create an encryption password
2. **Create habits**: Click "Add Habit" to create your first habit tracker
3. **Track progress**: Use the "✅ Today" button to mark habits as complete

### Managing Habits
- **View calendar**: Click on any habit name to expand its calendar view
- **Mark completion**: Click on any date in the calendar to toggle completion
- **Edit habit**: Use the "✏️ Edit" button to modify habit details
- **Delete habit**: Use the "🗑️ Delete" button to remove habits (with confirmation)

### Settings
Access settings via the ⚙️ button in the header:
- **Theme**: Toggle between light and dark modes
- **Change password**: Update your encryption password
- **Backup**: Export encrypted backups of your data
- **Restore**: Import data from encrypted backup files
- **Reset**: Delete all data and start fresh

## Technical Details

### Architecture
- **Frontend**: GTK4 with libadwaita for native Linux desktop integration
- **Backend**: Rust with secure encryption and local file storage
- **Data format**: JSON with AES-256-GCM encryption
- **Password hashing**: Argon2 with random salt generation

### Dependencies
- `gtk4` - GUI framework
- `libadwaita` - Modern GNOME styling
- `chrono` - Date and time handling
- `serde` - Data serialization
- `aes-gcm` - Encryption implementation  
- `argon2` - Password hashing
- `rand` - Cryptographic random number generation
- `dirs` - Cross-platform directory detection

### File Structure
```
src/
├── main.rs          # Application entry point
├── ui.rs            # Main UI components and event handling
├── habit.rs         # Habit data structures and logic
├── storage.rs       # Encrypted file storage
├── encryption.rs    # Cryptographic operations
├── calendar.rs      # Calendar widget implementation
└── style.css        # Custom CSS styling
```

## Security

HabitVault takes your privacy seriously:

- **No telemetry** - no data is sent to external servers
- **Local encryption** - all data is encrypted before being written to disk
- **Secure password handling** - passwords are never stored in plaintext
- **Memory safety** - built with Rust for memory-safe operations
- **Cryptographic standards** - uses industry-standard encryption algorithms

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

### Development Setup
```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential

# Clone and build
git clone https://github.com/codedsleep/habit-vault.git
cd habit-vault
cargo build
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [GTK4](https://gtk.org/) and [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- Encryption provided by [RustCrypto](https://github.com/RustCrypto)
- Icons and styling inspired by GNOME design guidelines
