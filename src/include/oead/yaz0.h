/**
 * Copyright (C) 2019 leoetlino <leo@leolam.fr>
 *
 * This file is part of syaz0.
 *
 * syaz0 is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 2 of the License, or
 * (at your option) any later version.
 *
 * syaz0 is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with syaz0.  If not, see <http://www.gnu.org/licenses/>.
 */

#pragma once

#include <array>
#include <nonstd/span.h>
#include <optional>
#include <vector>

#include <oead/types.h>
#include <oead/util/swap.h>
#include "rust/cxx.h"

namespace oead::yaz0 {

class Header {
public:
  /// 'Yaz0'
  std::array<char, 4> magic;
  /// Size of uncompressed data
  u32 uncompressed_size;
  /// [Newer files only] Required buffer alignment
  u32 data_alignment;
  /// Unused (as of December 2019)
  std::array<u8, 4> reserved;

  OEAD_DEFINE_FIELDS(Header, magic, uncompressed_size, data_alignment, reserved);
};
static_assert(sizeof(Header) == 0x10);

Header GetHeader(rust::Slice<const u8> data);

/// @param src  Source data
/// @param data_alignment  Required buffer alignment hint for decompression
/// @param level  Compression level (6 to 9; 6 is fastest and 9 is slowest)
rust::Vec<u8> Compress(rust::Slice<const u8> src, u32 data_alignment = 0, int level = 7);

void Decompress(rust::Slice<const u8> src, rust::Slice<u8> dst);
void DecompressUnsafe(rust::Slice<const u8> src, rust::Slice<u8> dst);

}  // namespace oead::yaz0
