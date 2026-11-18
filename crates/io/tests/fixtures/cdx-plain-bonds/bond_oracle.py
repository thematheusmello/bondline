# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent strict original CDX plain-bond reader; standard library only."""
import hashlib,json,struct,sys
LIMIT=8*1024*1024
HEADER=b'VjCD0100\x04\x03\x02\x01'+bytes(16)
NODE_FIELDS={0x0420:'isotope',0x0421:'charge',0x0422:'radical',0x042b:'explicit_hydrogens'}
BOND_FIELDS={0x0600:'order',0x0601:'display',0x0602:'display2',0x0603:'double_position'}
def check(ok,message):
    if not ok:raise ValueError(message)
def read(source):
    check(30<=len(source)<=LIMIT and source[:12]==HEADER[:12],'header/size')
    embedded=source[22:28]==struct.pack('<HI',0x8000,0)
    offset=22 if embedded else 28
    if not embedded:check(source[:28]==HEADER,'reserved header')
    explicit=struct.unpack_from('<H',source,offset)[0]==0x8000
    envelope='embedded' if embedded else 'explicit' if explicit else 'implied'
    records=[];stack=[]
    while True:
        check(len(records)<100000 and offset+2<=len(source),'truncated tag/record budget')
        start=offset;tag=struct.unpack_from('<H',source,offset)[0];offset+=2
        parent=stack[-1] if stack else None
        if tag==0:
            matched=stack.pop() if stack else None
            records.append({'kind':'end','parent':stack[-1] if stack else None,'matched':matched,'start':start,'end':offset})
            if not stack and (explicit or matched is None):
                if embedded:
                    check(source[offset:]==bytes(2),'distinct file end')
                    records.append({'kind':'file_end','parent':None,'start':offset,'end':offset+2});offset+=2
                check(offset==len(source),'trailing data');break
        elif tag&0x8000:
            check(offset+4<=len(source) and len(stack)<256,'object id/depth')
            ident=struct.unpack_from('<I',source,offset)[0];offset+=4
            records.append({'kind':'start','tag':tag,'source_id':ident,'parent':parent,'start':start,'end':offset});stack.append(len(records)-1)
        else:
            check(offset+2<=len(source),'short length');count=struct.unpack_from('<H',source,offset)[0];offset+=2;encoding='short'
            if count==65535:
                check(offset+4<=len(source),'extended length');count=struct.unpack_from('<I',source,offset)[0];offset+=4;encoding='extended'
            first=offset;offset+=count;check(offset<=len(source),'payload')
            records.append({'kind':'property','tag':tag,'parent':parent,'start':start,'end':offset,'encoding':encoding,'width':count,'payload_start':first,'payload_end':offset,'payload_hex':source[first:offset].hex()})
    def children(index):return [(i,r) for i,r in enumerate(records) if r.get('parent')==index and r['kind'] in ('start','property')]
    for i,r in enumerate(records):
        tag=r.get('tag');parent=records[r['parent']].get('tag') if r.get('parent') is not None else None
        if r['kind']=='start':
            if tag==0x8000:check(i==0 and parent is None,'Document containment')
            elif tag==0x8001:check(parent in (None,0x8000),'Page containment')
            elif tag in (0x8002,0x8003):check(parent in (0x8001,0x8002),'Group/Fragment containment')
            elif tag in (0x8004,0x8005):check(parent==0x8003,'Node/Bond containment')
            else:check(tag&0x4000,'unknown standard object')
        if tag in (*NODE_FIELDS,0x0200,0x0400,0x0402):check(parent==0x8004,'Node property owner')
        if tag in (*BOND_FIELDS,0x0604,0x0605):check(parent==0x8005,'Bond property owner')
        if r['kind']=='property' and parent not in (0x8004,0x8005):check(tag>=0x4000,'unsupported container property')
    fragments=[];atoms=[];bonds=[];source_to_model={}
    for index,r in enumerate(records):
        if r['kind']!='start' or r['tag']!=0x8003:continue
        source_to_model={}
        fragment={'source_id':r['source_id'],'model_id':len(fragments)+1,'atom_ids':[],'bond_ids':[]};fragments.append(fragment)
        items=children(index)
        for atom_index,node in items:
            if node['kind']!='start' or node['tag']!=0x8004:continue
            fields={}
            for i,field in children(atom_index):
                check(field['kind']=='property','unsupported Node child')
                tag=field['tag']
                if tag in (0x0200,0x0400,0x0402,*NODE_FIELDS):
                    check(tag not in fields,'duplicate known Node property');fields[tag]=(i,field)
                else:check(tag>=0x4000,'unknown standard Node property')
            def value(tag,fmt,default):
                if tag not in fields:return default
                _,record=fields[tag];check(record['width']==struct.calcsize(fmt),'numeric width')
                return struct.unpack(fmt,bytes.fromhex(record['payload_hex']))[0]
            check(value(0x0400,'<h',1)==1,'Element node type')
            element=value(0x0402,'<h',6);check(1<=element<=118,'element')
            point=None
            if 0x0200 in fields:
                _,record=fields[0x0200];check(record['width']==8,'point width');y,x=struct.unpack('<ii',bytes.fromhex(record['payload_hex']));point={'x_pt':x/65536,'y_pt':y/65536}
            projected={'isotope':None,'charge':0,'radical':'none','explicit_hydrogens':None};properties={}
            for tag,name in NODE_FIELDS.items():
                if tag not in fields:properties[name]={'present':False};continue
                i,record=fields[tag];payload=bytes.fromhex(record['payload_hex']);n=record['width'];extra={}
                if tag==0x0420:
                    check(n==2,'isotope width');raw=struct.unpack('<h',payload)[0];check(raw>=0,'isotope domain');projected[name]=raw or None
                elif tag==0x0421:
                    check(n in (1,4),'charge width')
                    if n==1:raw=struct.unpack('<b',payload)[0]
                    else:extra['saved_i32']=struct.unpack('<i',payload)[0];raw=extra['saved_i32']>>24
                    projected[name]=raw
                elif tag==0x0422:
                    check(n==1,'radical width');raw=payload[0];check(raw<=3,'radical enum');projected[name]=['none','singlet','doublet','triplet'][raw]
                else:check(n==2,'H width');raw=struct.unpack('<H',payload)[0];projected[name]=raw
                properties[name]={'present':True,'tag':tag,'width':n,'encoding':record['encoding'],'payload_hex':record['payload_hex'],'value':raw,'record_index':i,**extra}
            atom={'source_id':node['source_id'],'model_id':len(atoms)+1,'source_fragment_id':r['source_id'],'element':element,'position':point,**projected,'properties':properties}
            check(node['source_id'] not in source_to_model,'source atom identity');source_to_model[node['source_id']]=atom['model_id'];atoms.append(atom);fragment['atom_ids'].append(atom['model_id'])
        for bond_index,bond in items:
            if bond['kind']!='start' or bond['tag']!=0x8005:continue
            fields={}
            for i,record in children(bond_index):
                check(record['kind']=='property','bond child');tag=record['tag']
                if tag in (*BOND_FIELDS,0x0604,0x0605):
                    check(tag not in fields,'duplicate bond field');fields[tag]=(i,record)
                else:check(tag>=0x4000,'unsupported bond field')
            def bv(tag,fmt,default=None):
                if tag not in fields:return default
                _,record=fields[tag];check(record['width']==struct.calcsize(fmt),'bond field width');return struct.unpack(fmt,bytes.fromhex(record['payload_hex']))[0]
            order=bv(0x0600,'<H',1);check(order in (1,2,4),'unsupported order bitmask')
            display=bv(0x0601,'<h',0);check(display==0,'unsupported Display')
            display2=bv(0x0602,'<h',display);check(display2==0,'unsupported Display2')
            position=bv(0x0603,'<h');check(position in (None,0,1,2,256,257,258),'unsupported position')
            check(order==2 or 0x0602 not in fields and 0x0603 not in fields,'unsupported non-Double metadata')
            begin,end=bv(0x0604,'<I'),bv(0x0605,'<I')
            check(begin in source_to_model and end in source_to_model and begin!=end,'bond endpoints')
            properties={}
            for tag,name in BOND_FIELDS.items():
                if tag not in fields:properties[name]={'present':False};continue
                i,record=fields[tag];value=struct.unpack('<H' if tag==0x0600 else '<h',bytes.fromhex(record['payload_hex']))[0]
                properties[name]={'present':True,'tag':tag,'width':record['width'],'encoding':record['encoding'],'payload_hex':record['payload_hex'],'value':value,'record_index':i}
            side='unspecified' if position is None else {0:'center',1:'right',2:'left'}[position&255]
            result={'source_id':bond['source_id'],'model_id':len(bonds)+1,'source_fragment_id':r['source_id'],'begin_source':begin,'end_source':end,'begin':source_to_model[begin],'end':source_to_model[end],'order':{1:'single',2:'double',4:'triple'}[order],'double_position':side,'position_manual':None if position is None else position>=256,'effective_display':display,'effective_display2':display2,'properties':properties}
            bonds.append(result);fragment['bond_ids'].append(result['model_id'])
    check(atoms and fragments,'original atom graph')
    model_fragments=[]
    for fragment in fragments:
        model_atoms=[];model_bonds=[]
        for ident in fragment['atom_ids']:
            atom=next(a for a in atoms if a['model_id']==ident)
            model_atoms.append({'id':ident,**{k:atom[k] for k in ['element','position','isotope','charge','radical','explicit_hydrogens']}})
        for ident in fragment['bond_ids']:
            bond=next(b for b in bonds if b['model_id']==ident)
            model_bond={'id':ident,**{k:bond[k] for k in ['begin','end','order']}}
            if bond['double_position']!='unspecified':model_bond['double_position']=bond['double_position']
            model_bonds.append(model_bond)
        model_fragments.append({'id':fragment['model_id'],'atoms':model_atoms,'bonds':model_bonds})
    model={'pages':[{'id':1,'size':{'width_pt':210*72/25.4,'height_pt':297*72/25.4},'fragments':model_fragments}]}
    return {'envelope':envelope,'byte_length':len(source),'sha256':hashlib.sha256(source).hexdigest(),'records':records,'fragments':fragments,'atoms':atoms,'bonds':bonds,'model':model}
if __name__=='__main__':print(json.dumps(read(sys.stdin.buffer.read(LIMIT+1)),separators=(',',':')))
