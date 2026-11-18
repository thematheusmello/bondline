# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original fixed plain-CDX-bond refusal controls; standard library only."""
import struct

def negative_sources(sources,facts):
 base=sources['manual_right'];records=facts['manual_right']['records'];controls={};expected={}
 def add(name,raw,error):
  assert raw!=base;controls[name]=raw;expected[name]=error
 def property_bytes(tag,payload):return struct.pack('<HH',tag,len(payload))+payload
 def replace(record,payload):return base[:record['start']]+property_bytes(record['tag'],payload)+base[record['end']:]
 def record(tag):return next(r for r in records if r.get('tag')==tag)
 for tag in [0x0600,0x0601,0x0602,0x0603]:
  for width in [0,1,3,4,8]:add(f'width_{tag:04x}_{width}',replace(record(tag),bytes(width)),'InvalidProperty' if tag==0x0600 else 'UnsupportedProperty')
  for payload in [bytes.fromhex(record(tag)['payload_hex']),struct.pack('<h',1)]:
   r=record(tag);name=f'duplicate_{tag:04x}_'+('same' if payload.hex()==r['payload_hex'] else 'different');add(name,base[:r['end']]+property_bytes(tag,payload)+base[r['end']:],'DuplicateProperty')
 for value in [0,3,5,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65535]:add(f'unsupported_order_{value:04x}',replace(record(0x0600),struct.pack('<H',value)),'UnsupportedBondOrder')
 for tag in [0x0601,0x0602]:
  for value in [*range(1,15),15,-1,-32768,32767]:add(f'unsupported_style_{tag:04x}_{value}',replace(record(tag),struct.pack('<h',value)),'UnsupportedProperty')
 for value in [-32768,-1,3,255,259,32767]:add(f'unsupported_position_{value}',replace(record(0x0603),struct.pack('<h',value)),'UnsupportedProperty')
 for tag in [0x0600,0x0601,0x0602,0x0603]:
  payload=struct.pack('<h',2 if tag==0x0600 else 0);field=property_bytes(tag,payload)
  # Header-implied root is a distinct owner from the encoded Document object.
  implied=sources['single_absent'];add(f'wrong_owner_root_{tag:04x}',implied[:28]+field+implied[28:],'UnsupportedProperty')
  for owner in [0x8000,0x8001,0x8003,0x8004,0xc123]:
   at=record(owner)['end'];add(f'wrong_owner_{owner:04x}_{tag:04x}',base[:at]+field+base[at:],'UnsupportedProperty')
  at=record(0x8001)['end'];group=struct.pack('<HI',0x8002,901)+field+bytes(2);add(f'wrong_owner_group_{tag:04x}',base[:at]+group+base[at:],'UnsupportedProperty')
 for source_name in ['single_solid','ethyne']:
  raw=sources[source_name];table=facts[source_name]['records'];at=next(r['end'] for r in table if r.get('tag')==0x8005)
  for tag,values in [(0x0602,[0]),(0x0603,[0,1,2,256,257,258])]:
   for value in values:add(f'non_double_{source_name}_{tag:04x}_{value}',raw[:at]+property_bytes(tag,struct.pack('<h',value))+raw[at:],'UnsupportedProperty')
 for count in [1,2,3]:add(f'truncated_eof_{count}',base[:-count],'Syntax')
 for tag in [0x0600,0x0601,0x0602,0x0603]:
  r=record(tag)
  for count in [1,3,5]:add(f'truncated_field_{tag:04x}_{count}',base[:r['start']+count],'Syntax')
 raw=sources['extended_right'];r=next(r for r in facts['extended_right']['records'] if r.get('tag')==0x0603)
 for count in [1,3,5,7,9]:add(f'truncated_extended_{count}',raw[:r['start']+count],'Syntax')
 return {name:(raw,expected[name]) for name,raw in controls.items()}
