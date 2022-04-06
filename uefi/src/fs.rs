use {
    crate::{
        error::{ErrorKind, Result},
        io::{default_read_to_end, Initializer, Read},
    },
    alloc::vec::Vec,
    uefi::{
        data_types::CStr16,
        proto::media::{
            file::{
                Directory, File as EFIFileTrit, FileAttribute, FileInfo, FileMode, FileType,
                RegularFile,
            },
            fs::SimpleFileSystem,
        },
        table::boot::BootServices,
    },
};

pub(crate) struct File {
    cur: usize,
    file: RegularFile,
}

impl File {
    pub(crate) fn open(bs: &BootServices, filename: &str, open_mode: FileMode) -> Result<Self> {
        let mut root = root_dir(bs)?;

        let attributes = match open_mode {
            FileMode::CreateReadWrite => FileAttribute::VALID_ATTR,
            _ => {
                let mut buf = vec![0; 1 << 8];
                match root.get_info::<FileInfo>(&mut buf) {
                    Ok(info) => info.attribute(),
                    // TODO: hint the file size
                    Err(e) => return Err(e.status().into()),
                }
            }
        };

        let mut buf = [0; 1024];
        return match root.open(
            CStr16::from_str_with_buf(filename, &mut buf).unwrap(),
            open_mode,
            attributes,
        ) {
            Ok(handle) => match handle.into_type().unwrap() {
                FileType::Regular(file) => Ok(Self { cur: 0, file }),
                FileType::Dir(_) => Err(ErrorKind::IsADirectory.into()),
            },
            Err(e) => Err(e.status().into()),
        };
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.file.read(buf) {
            Ok(size) => {
                self.cur += size;
                Ok(size)
            }
            Err(e) => match e.data() {
                // TODO: hint the file size
                Some(_size) => Err(e.status().into()),
                None => Err(e.status().into()),
            },
        }
    }

    #[inline]
    unsafe fn initializer(&self) -> Initializer {
        // SAFETY: Read is guaranteed to work on uninitialized memory
        Initializer::nop()
    }

    // Reserves space in the buffer based on the file size when available.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut buffer = vec![0; 1 << 8];
        if let Ok(info) = self.file.get_info::<FileInfo>(&mut buffer) {
            buf.reserve(info.file_size() as usize);
        }
        default_read_to_end(self, buf)
    }
}

/// Get the root directory of the file system
fn root_dir(bs: &BootServices) -> Result<Directory> {
    let fs = match bs.locate_protocol::<SimpleFileSystem>() {
        Ok(fs) => fs,
        Err(e) => return Err(e.status().into()),
    };
    let fs = unsafe { &mut *fs.get() };
    match fs.open_volume() {
        Ok(root) => Ok(root),
        Err(e) => Err(e.status().into()),
    }
}
