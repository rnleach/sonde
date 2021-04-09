#! /usr/bin/bash -l

PROG="sonde.exe"
DIST_ZIP="sonde.zip"

MINGW_PREFIX="/mingw64"
PIX_BUF="lib/gdk-pixbuf-2.0/2.10.0/loaders"
PACKAGE_DIR="dist"
TARGET="target/release"

echo "Cleaning up: ${PACKAGE_DIR} ${DIST_ZIP}"
rm -rf ${PACKAGE_DIR} ${DIST_ZIP}

echo "Making sure binary is up to date."
RUSTFLAGS="-C target-cpu=generic" cargo build --release

echo "Create distribution folder: ${PACKAGE_DIR}"
mkdir ${PACKAGE_DIR}

echo "Copy program: ${TARGET}/${PROG} to ${PACKAGE_DIR}/${PROG}"
cp ${TARGET}/${PROG} ${PACKAGE_DIR}/

echo "Copy DLLs"
cp ${MINGW_PREFIX}/bin/*.dll ${PACKAGE_DIR}/

echo "Copy schemas"
mkdir -p ${PACKAGE_DIR}/share/glib-2.0/schemas
cp -r ${MINGW_PREFIX}/share/glib-2.0/schemas ${PACKAGE_DIR}/share/glib-2.0/

echo "Set up pixbuf to work in the new environment"
mkdir -p ${PACKAGE_DIR}/${PIX_BUF}
cp ${MINGW_PREFIX}/${PIX_BUF}/*.dll ${PACKAGE_DIR}/${PIX_BUF}/
cd ${PACKAGE_DIR}
GKD_PIXBUF_MODULEDIR=${PIX_BUF} gdk-pixbuf-query-loaders > ${PIX_BUF}/../loaders.cache
cd ..

echo "Copy icons to distribution folder"
cp -r ${MINGW_PREFIX}/share/icons ${PACKAGE_DIR}/share/icons

echo "Copy graphics"
cp -r graphics ${PACKAGE_DIR}/

echo "Zipping up into ${DIST_ZIP}"
cd ${PACKAGE_DIR}
zip -ruqo9 ../${DIST_ZIP} ./*
cd ..
